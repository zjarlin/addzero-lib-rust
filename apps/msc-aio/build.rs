use std::{
    env, fs,
    path::{Path, PathBuf},
};

use addzero_knowledge::{
    KnowledgeService, KnowledgeSourceSpec, database_url, discover_documents, local_env_path,
    render_catalog, source_specs,
};

fn main() {
    println!("cargo:rerun-if-env-changed=DIOXUS_ADMIN_KB_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=DATABASE_URL");
    println!("cargo:rerun-if-env-changed=MSC_AIO_DATABASE_URL");
    println!("cargo:rerun-if-env-changed=MSC_AIO_KNOWLEDGE_EXTRA_ROOTS");
    println!("cargo:rerun-if-changed=assets/app-icon.png");

    if let Some(path) = local_env_path() {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let sources = build_sources();
    for source in &sources {
        println!("cargo:rerun-if-changed={}", source.root_path.display());
    }

    let (mode, docs) = load_docs(&sources);
    let output = render_catalog(&mode, &sources, &docs);
    let out_path =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("knowledge_catalog.rs");

    fs::write(out_path, output).expect("failed to write generated knowledge catalog");
    write_app_icon_rgba();
}

fn write_app_icon_rgba() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let icon_path = Path::new("assets/app-icon.png");
    let file = fs::File::open(icon_path).expect("failed to open app icon");
    let decoder = png::Decoder::new(file);
    let mut reader = decoder
        .read_info()
        .expect("failed to read app icon metadata");
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .expect("failed to decode app icon");
    let bytes = &buffer[..info.buffer_size()];

    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => bytes.to_vec(),
        (png::ColorType::Rgb, png::BitDepth::Eight) => bytes
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        (color, depth) => panic!("unsupported app icon format: {color:?} {depth:?}"),
    };

    let metadata = format!(
        "pub const APP_ICON_WIDTH: u32 = {};\npub const APP_ICON_HEIGHT: u32 = {};\n",
        info.width, info.height
    );
    fs::write(out_dir.join("app_icon.rgba"), rgba).expect("failed to write decoded app icon");
    fs::write(out_dir.join("app_icon.rs"), metadata).expect("failed to write app icon metadata");
}

fn load_docs(
    sources: &[KnowledgeSourceSpec],
) -> (String, Vec<addzero_knowledge::KnowledgeDocument>) {
    let Some(url) = database_url() else {
        let scan = discover_documents(sources);
        return ("filesystem-fallback".to_string(), scan.documents);
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime should be available");

    runtime.block_on(async {
        match KnowledgeService::connect(&url).await {
            Ok(service) => {
                if let Err(err) = service.sync_sources(sources).await {
                    eprintln!("knowledge sync failed, falling back to pg snapshot: {err}");
                }

                match service.list_documents().await {
                    Ok(docs) if !docs.is_empty() => ("postgres-sync".to_string(), docs),
                    Ok(_) => {
                        let scan = discover_documents(sources);
                        ("filesystem-fallback".to_string(), scan.documents)
                    }
                    Err(err) => {
                        eprintln!("knowledge list failed, falling back to filesystem: {err}");
                        let scan = discover_documents(sources);
                        ("filesystem-fallback".to_string(), scan.documents)
                    }
                }
            }
            Err(err) => {
                eprintln!("knowledge pg attach failed, falling back to filesystem: {err}");
                let scan = discover_documents(sources);
                ("filesystem-fallback".to_string(), scan.documents)
            }
        }
    })
}

fn build_sources() -> Vec<KnowledgeSourceSpec> {
    let mut sources = source_specs();
    sources.extend(download_station_sources());
    sources.sort_by(|left, right| left.name.cmp(&right.name));
    sources.dedup_by(|left, right| left.root_path == right.root_path);
    sources
}

fn download_station_sources() -> Vec<KnowledgeSourceSpec> {
    let Some(home) = env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
    else {
        return Vec::new();
    };

    let candidates = [
        (
            "download-station-yesterday-research",
            "Download Station / 昨天研究成果",
            home.join("Desktop/昨天研究成果"),
        ),
        (
            "download-station-desktop-research",
            "Download Station / 桌面研究成果",
            home.join("Desktop/research-results"),
        ),
        (
            "download-station-cron-output",
            "Download Station / Hermes 输出",
            home.join(".hermes/cron/output"),
        ),
    ];

    candidates
        .into_iter()
        .filter(|(_, _, path)| path.exists())
        .map(|(slug, name, path)| KnowledgeSourceSpec::new(slug, name, path))
        .collect()
}
