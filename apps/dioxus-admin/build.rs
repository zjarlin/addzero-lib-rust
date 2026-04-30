use std::{env, fs, path::PathBuf};

use addzero_knowledge::{
    KnowledgeService, database_url, discover_documents, local_env_path, render_catalog,
    source_specs,
};

fn main() {
    println!("cargo:rerun-if-env-changed=DIOXUS_ADMIN_KB_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=DATABASE_URL");
    println!("cargo:rerun-if-env-changed=MSC_AIO_DATABASE_URL");
    println!("cargo:rerun-if-env-changed=MSC_AIO_KNOWLEDGE_EXTRA_ROOTS");

    if let Some(path) = local_env_path() {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let sources = source_specs();
    for source in &sources {
        println!("cargo:rerun-if-changed={}", source.root_path.display());
    }

    let (mode, docs) = load_docs(&sources);
    let output = render_catalog(&mode, &sources, &docs);
    let out_path =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("knowledge_catalog.rs");

    fs::write(out_path, output).expect("failed to write generated knowledge catalog");
}

fn load_docs(
    sources: &[addzero_knowledge::KnowledgeSourceSpec],
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
