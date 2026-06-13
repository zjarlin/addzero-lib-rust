#![forbid(unsafe_code)]

use std::{
    env, fmt, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use az_aio_plugin_api::{
    ContributionSet, PluginBackendBundle, PluginBundleArtifact, PluginBundleArtifactKind,
    PluginBundleManifest, PluginBundleSandbox, PluginDescriptor, PluginFrontendBundle,
    PluginSandboxDebugReport, PluginState,
};
use serde::{Deserialize, Serialize};

const PLUGINS_MANIFEST: &str = "apps/az-aio/plugins/Cargo.toml";
const AZ_PLATFORM: &str = "az-platform";
const PLUGIN_MANIFEST_FILE: &str = "az-plugin.json";
const FRONTEND_BUNDLE_FILE: &str = "frontend/az-frontend.json";
const BACKEND_BUNDLE_FILE: &str = "backend/az-backend.json";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), XtaskError> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        let message = usage();
        let error = XtaskError::Usage(message);

        return Err(error);
    }

    match args.remove(0).as_str() {
        "az-platform" => run_az_platform(args),
        other => Err(XtaskError::Usage(format!(
            "unknown xtask command `{other}`\n\n{}",
            usage()
        ))),
    }
}

fn run_az_platform(mut args: Vec<String>) -> Result<(), XtaskError> {
    if args.first().map(String::as_str) != Some("plugin") {
        let message = az_platform_usage();
        let error = XtaskError::Usage(message);

        return Err(error);
    }
    args.remove(0);

    let Some(command) = args.first().map(String::as_str) else {
        let message = az_platform_usage();
        let error = XtaskError::Usage(message);

        return Err(error);
    };
    match command {
        "list" => {
            print_plugin_list();
            Ok(())
        }
        "build" => run_plugin_build_arg(&args),
        "build-wasm" => {
            if args.get(1).map(String::as_str) == Some("all") {
                for plugin in PLUGIN_TARGETS {
                    run_plugin_wasm_build(plugin)?;
                }
                Ok(())
            } else {
                let plugin = required_plugin_arg(&args)?;
                run_plugin_wasm_build(plugin).map(|_| ())
            }
        }
        "package" => package_plugins(&args),
        "sandbox" => run_sandbox_arg(&args),
        _ => Err(XtaskError::Usage(az_platform_usage())),
    }
}

fn package_plugins(args: &[String]) -> Result<(), XtaskError> {
    let Some(name) = args.get(1) else {
        let message = az_platform_usage();
        let error = XtaskError::Usage(message);

        return Err(error);
    };
    if name == "all" {
        for plugin in PLUGIN_TARGETS {
            run_plugin_package(plugin)?;
        }
        return Ok(());
    }
    let plugin =
        plugin_target(name).ok_or_else(|| XtaskError::Usage(format!("unknown plugin `{name}`")))?;
    run_plugin_package(plugin)
}

fn run_plugin_build_arg(args: &[String]) -> Result<(), XtaskError> {
    let Some(name) = args.get(1) else {
        let message = az_platform_usage();
        let error = XtaskError::Usage(message);

        return Err(error);
    };
    if name == "all" {
        for plugin in PLUGIN_TARGETS {
            run_plugin_cargo_command("build", plugin)?;
        }
        return Ok(());
    }
    let plugin =
        plugin_target(name).ok_or_else(|| XtaskError::Usage(format!("unknown plugin `{name}`")))?;
    run_plugin_cargo_command("build", plugin)
}

fn run_sandbox_arg(args: &[String]) -> Result<(), XtaskError> {
    let Some(name_or_manifest) = args.get(1) else {
        let message = az_platform_usage();
        let error = XtaskError::Usage(message);

        return Err(error);
    };
    if name_or_manifest == "all" {
        for plugin in PLUGIN_TARGETS {
            run_plugin_package(plugin)?;
            run_plugin_bundle_sandbox(&plugin_bundle_dir(plugin).join(PLUGIN_MANIFEST_FILE))?;
        }
        return Ok(());
    }
    if let Some(plugin) = plugin_target(name_or_manifest) {
        run_plugin_package(plugin)?;
        return run_plugin_bundle_sandbox(&plugin_bundle_dir(plugin).join(PLUGIN_MANIFEST_FILE));
    }
    run_plugin_bundle_sandbox(Path::new(name_or_manifest))
}

fn required_plugin_arg(args: &[String]) -> Result<&'static PluginTarget, XtaskError> {
    let Some(name) = args.get(1) else {
        let message = az_platform_usage();
        let error = XtaskError::Usage(message);

        return Err(error);
    };
    if name == "all" {
        let message = "`build-wasm all` is handled as a command target, not a plugin".to_string();
        let error = XtaskError::Usage(message);

        return Err(error);
    }
    plugin_target(name).ok_or_else(|| XtaskError::Usage(format!("unknown plugin `{name}`")))
}

fn run_plugin_cargo_command(command: &str, plugin: &PluginTarget) -> Result<(), XtaskError> {
    let repo_root = repo_root();
    let status = Command::new("cargo")
        .arg(command)
        .arg("--manifest-path")
        .arg(repo_root.join(PLUGINS_MANIFEST))
        .arg("-p")
        .arg(plugin.package)
        .env("CARGO_TARGET_DIR", target_root())
        .status()
        .map_err(XtaskError::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(XtaskError::Command {
            command: format!("cargo {command} -p {}", plugin.package),
            status,
        })
    }
}

fn run_plugin_package(plugin: &PluginTarget) -> Result<(), XtaskError> {
    let component_path = run_plugin_wasm_build(plugin)?;
    let mut artifacts = vec![PluginBundleArtifact {
        kind: PluginBundleArtifactKind::WasmComponent,
        name: format!("{}.component.wasm", plugin.package),
        source: plugin.source_path.to_string(),
        path: Some(component_path.display().to_string()),
    }];
    let snapshot = load_wasm_sandbox_snapshot(plugin.name, &component_path)?;
    let bundle_dir = plugin_bundle_dir(plugin);
    let frontend_bundle_path = write_frontend_bundle(plugin, &snapshot.contributions, &bundle_dir)?;
    artifacts.push(PluginBundleArtifact {
        kind: PluginBundleArtifactKind::FrontendBundle,
        name: "az-frontend.json".to_string(),
        source: plugin.source_path.to_string(),
        path: Some(frontend_bundle_path.display().to_string()),
    });
    let backend_bundle_path = write_backend_bundle(plugin, &snapshot.contributions, &bundle_dir)?;
    artifacts.push(PluginBundleArtifact {
        kind: PluginBundleArtifactKind::BackendBundle,
        name: "az-backend.json".to_string(),
        source: plugin.source_path.to_string(),
        path: Some(backend_bundle_path.display().to_string()),
    });
    let sandbox_debug = if snapshot.sandbox_debug == PluginSandboxDebugReport::default() {
        PluginSandboxDebugReport::from_contributions(&snapshot.contributions)
    } else {
        snapshot.sandbox_debug.clone()
    };
    let manifest = PluginBundleManifest {
        schema_version: PluginBundleManifest::SCHEMA_VERSION,
        platform: AZ_PLATFORM.to_string(),
        bundle_id: plugin.name.to_string(),
        package: plugin.package.to_string(),
        descriptor: snapshot.descriptor,
        contributions: snapshot.contributions,
        artifacts,
        sandbox_debug,
        sandbox: PluginBundleSandbox {
            command: vec![
                "cargo".to_string(),
                "xtask".to_string(),
                AZ_PLATFORM.to_string(),
                "plugin".to_string(),
                "sandbox".to_string(),
                plugin.name.to_string(),
            ],
        },
    };
    fs::create_dir_all(&bundle_dir).map_err(XtaskError::Io)?;
    let manifest_path = bundle_dir.join(PLUGIN_MANIFEST_FILE);
    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(XtaskError::Json)?;
    fs::write(&manifest_path, manifest_json).map_err(XtaskError::Io)?;
    println!("packaged {} -> {}", plugin.name, manifest_path.display());
    Ok(())
}

fn run_plugin_wasm_build(plugin: &PluginTarget) -> Result<PathBuf, XtaskError> {
    let Some(wasm_artifact_stem) = plugin.wasm_artifact_stem else {
        let message = format!(
            "plugin `{}` does not expose a wasm component target yet",
            plugin.name
        );
        let error = XtaskError::Usage(message);

        return Err(error);
    };
    let repo_root = repo_root();
    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(repo_root.join(PLUGINS_MANIFEST))
        .arg("-p")
        .arg(plugin.package)
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .env("CARGO_TARGET_DIR", target_root())
        .status()
        .map_err(XtaskError::Io)?;

    if !status.success() {
        let command = format!(
            "cargo build -p {} --target wasm32-unknown-unknown",
            plugin.package
        );
        let error = XtaskError::Command { command, status };

        return Err(error);
    }

    let core_wasm_path = target_root()
        .join("wasm32-unknown-unknown")
        .join("debug")
        .join(format!("{wasm_artifact_stem}.wasm"));
    let component_path =
        plugin_bundle_dir(plugin).join(format!("{}.component.wasm", plugin.package));
    let core_wasm = fs::read(&core_wasm_path).map_err(XtaskError::Io)?;
    let mut encoder = wit_component::ComponentEncoder::default()
        .module(&core_wasm)
        .map_err(|error| XtaskError::Component(error.to_string()))?
        .validate(true);
    let component = encoder
        .encode()
        .map_err(|error| XtaskError::Component(error.to_string()))?;
    if let Some(parent) = component_path.parent() {
        fs::create_dir_all(parent).map_err(XtaskError::Io)?;
    }
    fs::write(&component_path, component).map_err(XtaskError::Io)?;
    println!(
        "built wasm component {} -> {}",
        plugin.name,
        component_path.display()
    );
    Ok(component_path)
}

fn run_plugin_bundle_sandbox(path: &Path) -> Result<(), XtaskError> {
    let manifest_json = fs::read_to_string(path).map_err(XtaskError::Io)?;
    let manifest: PluginBundleManifest =
        serde_json::from_str(&manifest_json).map_err(XtaskError::Json)?;

    println!("az-platform packaged sandbox");
    println!("manifest: {}", path.display());
    println!("bundle: {}", manifest.bundle_id);
    println!("package: {}", manifest.package);
    println!("platform: {}", manifest.platform);
    println!();
    print_json_block("descriptor", &manifest.descriptor)?;
    println!();
    print_json_block("ui_contributions", &manifest.contributions.ui_contributions)?;
    println!();
    print_json_block("backend_apis", &manifest.contributions.backend_apis)?;
    println!();
    print_json_block("artifacts", &manifest.artifacts)?;
    println!();
    print_artifact_json(
        &manifest,
        PluginBundleArtifactKind::FrontendBundle,
        "frontend_bundle",
    )?;
    println!();
    print_artifact_json(
        &manifest,
        PluginBundleArtifactKind::BackendBundle,
        "backend_bundle",
    )?;
    println!();
    print_json_block("sandbox_debug", &manifest.sandbox_debug)?;
    println!();
    run_manifest_wasm_sandbox(&manifest)?;
    println!();
    print_json_block("sandbox", &manifest.sandbox)?;
    Ok(())
}

fn write_frontend_bundle(
    plugin: &PluginTarget,
    contributions: &ContributionSet,
    bundle_dir: &Path,
) -> Result<PathBuf, XtaskError> {
    let bundle = PluginFrontendBundle {
        schema_version: PluginFrontendBundle::SCHEMA_VERSION,
        plugin_id: plugin.name.to_string(),
        nav_items: contributions.nav_items.clone(),
        pages: contributions.pages.clone(),
        ui_contributions: contributions.ui_contributions.clone(),
        toolbar_actions: contributions.toolbar_actions.clone(),
        catalog_providers: contributions.catalog_providers.clone(),
        settings_sections: contributions.settings_sections.clone(),
    };
    write_json_artifact(bundle_dir, FRONTEND_BUNDLE_FILE, &bundle)
}

fn write_backend_bundle(
    plugin: &PluginTarget,
    contributions: &ContributionSet,
    bundle_dir: &Path,
) -> Result<PathBuf, XtaskError> {
    let bundle = PluginBackendBundle {
        schema_version: PluginBackendBundle::SCHEMA_VERSION,
        plugin_id: plugin.name.to_string(),
        backend_apis: contributions.backend_apis.clone(),
        shell_entries: contributions.shell_entries.clone(),
        generated_files: contributions.generated_files.clone(),
    };
    write_json_artifact(bundle_dir, BACKEND_BUNDLE_FILE, &bundle)
}

fn write_json_artifact<T: Serialize>(
    bundle_dir: &Path,
    relative_path: &str,
    value: &T,
) -> Result<PathBuf, XtaskError> {
    let path = bundle_dir.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(XtaskError::Io)?;
    }
    let json = serde_json::to_string_pretty(value).map_err(XtaskError::Json)?;
    fs::write(&path, json).map_err(XtaskError::Io)?;
    Ok(path)
}

fn print_artifact_json(
    manifest: &PluginBundleManifest,
    kind: PluginBundleArtifactKind,
    label: &str,
) -> Result<(), XtaskError> {
    let Some(path) = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == kind)
        .and_then(|artifact| artifact.path.as_ref())
    else {
        println!("{label}: null");
        return Ok(());
    };
    let json = fs::read_to_string(path).map_err(XtaskError::Io)?;
    let value = serde_json::from_str::<serde_json::Value>(&json).map_err(XtaskError::Json)?;
    print_json_block(label, &value)
}

fn run_manifest_wasm_sandbox(manifest: &PluginBundleManifest) -> Result<(), XtaskError> {
    let Some(path) = artifact_path(manifest, PluginBundleArtifactKind::WasmComponent) else {
        println!("wasm_runtime: null");
        return Ok(());
    };
    let snapshot = load_wasm_sandbox_snapshot(&manifest.bundle_id, Path::new(path))?;
    print_json_block("wasm_runtime", &snapshot)
}

fn artifact_path(manifest: &PluginBundleManifest, kind: PluginBundleArtifactKind) -> Option<&str> {
    manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == kind)
        .and_then(|artifact| artifact.path.as_deref())
}

fn print_json_block<T: Serialize>(label: &str, value: &T) -> Result<(), XtaskError> {
    let json = serde_json::to_string_pretty(value).map_err(XtaskError::Json)?;
    println!("{label}:");
    println!("{json}");
    Ok(())
}

fn load_wasm_sandbox_snapshot(
    plugin_id: &str,
    wasm_file: &Path,
) -> Result<PluginSandboxSnapshot, XtaskError> {
    let repo_root = repo_root();
    let output = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(repo_root.join(PLUGINS_MANIFEST))
        .arg("-p")
        .arg("az-aio-plugin-host")
        .arg("--example")
        .arg("sandbox")
        .arg("--")
        .arg("--json")
        .arg("--wasm-file")
        .arg(wasm_file)
        .arg(plugin_id)
        .env("CARGO_TARGET_DIR", target_root())
        .output()
        .map_err(XtaskError::Io)?;

    if !output.status.success() {
        let command = format!(
            "cargo run -p az-aio-plugin-host --example sandbox -- --json --wasm-file {} {}",
            wasm_file.display(),
            plugin_id,
        );
        let status = output.status;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let error = XtaskError::CommandOutput {
            command,
            status,
            stderr,
        };

        return Err(error);
    }

    serde_json::from_slice(&output.stdout).map_err(XtaskError::Json)
}

fn plugin_bundle_dir(plugin: &PluginTarget) -> PathBuf {
    plugin.name.split('/').fold(
        target_root().join(AZ_PLATFORM).join("plugins"),
        |path, part| path.join(part),
    )
}

fn target_root() -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("target"))
}

fn print_plugin_list() {
    for plugin in PLUGIN_TARGETS {
        println!("{}\t{}", plugin.name, plugin.package);
    }
}

fn plugin_target(name: &str) -> Option<&'static PluginTarget> {
    PLUGIN_TARGETS
        .iter()
        .find(|plugin| plugin.name == name || plugin.aliases.contains(&name))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn usage() -> String {
    format!("usage:\n\n{az_platform}", az_platform = az_platform_usage())
}

fn az_platform_usage() -> String {
    "cargo xtask az-platform plugin list\ncargo xtask az-platform plugin build <plugin|all>\ncargo xtask az-platform plugin build-wasm <plugin|all>\ncargo xtask az-platform plugin package <plugin|all>\ncargo xtask az-platform plugin sandbox <plugin|all|az-plugin.json>".to_string()
}

struct PluginTarget {
    name: &'static str,
    package: &'static str,
    source_path: &'static str,
    wasm_artifact_stem: Option<&'static str>,
    aliases: &'static [&'static str],
}

const PLUGIN_TARGETS: &[PluginTarget] = &[
    PluginTarget {
        name: "navigation",
        package: "az-aio-plugin-core-nav",
        source_path: "apps/az-aio/plugins/features/navigation",
        wasm_artifact_stem: Some("az_aio_plugin_core_nav"),
        aliases: &["core-nav"],
    },
    PluginTarget {
        name: "catalog",
        package: "az-aio-plugin-catalog",
        source_path: "apps/az-aio/plugins/features/catalog",
        wasm_artifact_stem: Some("az_aio_plugin_catalog"),
        aliases: &[],
    },
    PluginTarget {
        name: "settings",
        package: "az-aio-plugin-settings",
        source_path: "apps/az-aio/plugins/features/settings",
        wasm_artifact_stem: Some("az_aio_plugin_settings"),
        aliases: &[],
    },
    PluginTarget {
        name: "search",
        package: "az-aio-plugin-search",
        source_path: "apps/az-aio/plugins/features/search",
        wasm_artifact_stem: Some("az_aio_plugin_search"),
        aliases: &[],
    },
    PluginTarget {
        name: "projects",
        package: "az-aio-plugin-projects",
        source_path: "apps/az-aio/plugins/features/projects",
        wasm_artifact_stem: Some("az_aio_plugin_projects"),
        aliases: &[],
    },
    PluginTarget {
        name: "sync",
        package: "az-aio-plugin-sync",
        source_path: "apps/az-aio/plugins/features/sync",
        wasm_artifact_stem: Some("az_aio_plugin_sync"),
        aliases: &[],
    },
    PluginTarget {
        name: "lowcode",
        package: "az-aio-plugin-lowcode",
        source_path: "apps/az-aio/plugins/features/lowcode",
        wasm_artifact_stem: Some("az_aio_plugin_lowcode"),
        aliases: &["low-code"],
    },
    PluginTarget {
        name: "git/skills",
        package: "az-aio-plugin-git-skills",
        source_path: "apps/az-aio/plugins/features/git/skills",
        wasm_artifact_stem: Some("az_aio_plugin_git_skills"),
        aliases: &["skills"],
    },
    PluginTarget {
        name: "git/clis",
        package: "az-aio-plugin-git-clis",
        source_path: "apps/az-aio/plugins/features/git/clis",
        wasm_artifact_stem: Some("az_aio_plugin_git_clis"),
        aliases: &["clis"],
    },
    PluginTarget {
        name: "git/envs",
        package: "az-aio-plugin-git-envs",
        source_path: "apps/az-aio/plugins/features/git/envs",
        wasm_artifact_stem: Some("az_aio_plugin_git_envs"),
        aliases: &["envs"],
    },
    PluginTarget {
        name: "git/notes",
        package: "az-aio-plugin-git-notes",
        source_path: "apps/az-aio/plugins/features/git/notes",
        wasm_artifact_stem: Some("az_aio_plugin_git_notes"),
        aliases: &["notes"],
    },
];

#[derive(Debug, Deserialize, Serialize)]
struct PluginSandboxSnapshot {
    descriptor: PluginDescriptor,
    #[serde(default)]
    state: Option<PluginState>,
    contributions: ContributionSet,
    #[serde(default)]
    sandbox_debug: PluginSandboxDebugReport,
}

#[derive(Debug)]
enum XtaskError {
    Usage(String),
    Io(io::Error),
    Json(serde_json::Error),
    Component(String),
    Command {
        command: String,
        status: std::process::ExitStatus,
    },
    CommandOutput {
        command: String,
        status: std::process::ExitStatus,
        stderr: String,
    },
}

impl fmt::Display for XtaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}"),
            Self::Io(error) => write!(formatter, "xtask io error: {error}"),
            Self::Json(error) => write!(formatter, "xtask json error: {error}"),
            Self::Component(error) => write!(formatter, "xtask component error: {error}"),
            Self::Command { command, status } => {
                write!(formatter, "`{command}` failed with {status}")
            }
            Self::CommandOutput {
                command,
                status,
                stderr,
            } => {
                write!(formatter, "`{command}` failed with {status}")?;
                if stderr.trim().is_empty() {
                    Ok(())
                } else {
                    write!(formatter, "\n{}", stderr.trim())
                }
            }
        }
    }
}
