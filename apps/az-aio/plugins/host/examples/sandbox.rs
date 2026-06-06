#![forbid(unsafe_code)]

use std::{env, path::PathBuf, process::ExitCode};

use az_aio_plugin_api::PluginSandboxDebugReport;
use az_aio_plugin_host::{
    PluginHost, PluginRuntimeRecord, WasmComponentPlugin, default_plugin_host,
};
use serde::Serialize;

#[derive(Clone, Copy)]
enum OutputMode {
    Text,
    Json,
}

#[derive(Serialize)]
struct SandboxSnapshot {
    descriptor: az_aio_plugin_api::PluginDescriptor,
    state: az_aio_plugin_api::PluginState,
    contributions: az_aio_plugin_api::ContributionSet,
    sandbox_debug: PluginSandboxDebugReport,
}

struct SandboxArgs {
    output_mode: OutputMode,
    wasm_file: Option<PathBuf>,
    plugin_id: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1).collect())?;
    let snapshot = if let Some(wasm_file) = args.wasm_file.as_ref() {
        let plugin =
            WasmComponentPlugin::from_file(wasm_file).map_err(|error| error.to_string())?;
        PluginHost::new()
            .with_plugin(Box::new(plugin))
            .load_snapshot()
    } else {
        default_plugin_host().load_snapshot()
    };
    let plugin_id = args
        .plugin_id
        .clone()
        .or_else(|| {
            snapshot
                .plugins
                .first()
                .map(|record| record.descriptor.id.clone())
        })
        .ok_or_else(|| "usage: sandbox [--json] [--wasm-file <path>] <plugin-id>".to_string())?;
    let record = snapshot
        .plugins
        .iter()
        .find(|record| record.descriptor.id == plugin_id)
        .ok_or_else(|| unknown_plugin_message(&plugin_id, &snapshot.plugins))?;
    let contributions = snapshot
        .plugin_contributions
        .iter()
        .find(|contribution| contribution.plugin_id == plugin_id)
        .map(|contribution| contribution.contributions.clone())
        .unwrap_or_default();
    let sandbox_debug = PluginSandboxDebugReport::from_contributions(&contributions);

    if matches!(args.output_mode, OutputMode::Json) {
        print_json(&SandboxSnapshot {
            descriptor: record.descriptor.clone(),
            state: record.state.clone(),
            contributions,
            sandbox_debug,
        })?;
        return Ok(());
    }

    println!("az-platform sandbox");
    println!("plugin: {}", record.descriptor.id);
    println!("state: {:?}", record.state);
    if let Some(error) = record.error.as_deref() {
        println!("error: {error}");
    }
    println!();
    println!("descriptor:");
    print_json(&record.descriptor)?;
    println!();
    println!("ui_contributions:");
    print_json(&contributions.ui_contributions)?;
    println!();
    println!("backend_apis:");
    print_json(&contributions.backend_apis)?;
    println!();
    println!("settings_sections:");
    print_json(&contributions.settings_sections)?;
    println!();
    println!("sandbox_debug:");
    print_json(&sandbox_debug)?;
    println!();
    println!("shell_entries: {}", contributions.shell_entries.len());
    println!("generated_files: {}", contributions.generated_files.len());
    println!(
        "catalog_providers: {}",
        contributions.catalog_providers.len()
    );

    Ok(())
}

fn parse_args(mut args: Vec<String>) -> Result<SandboxArgs, String> {
    let mut output_mode = OutputMode::Text;
    let mut wasm_file = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                output_mode = OutputMode::Json;
                args.remove(index);
            }
            "--wasm-file" => {
                args.remove(index);
                let Some(path) = args.get(index).cloned() else {
                    return Err(
                        "usage: sandbox [--json] [--wasm-file <path>] <plugin-id>".to_string()
                    );
                };
                args.remove(index);
                wasm_file = Some(PathBuf::from(path));
            }
            _ => index += 1,
        }
    }

    Ok(SandboxArgs {
        output_mode,
        wasm_file,
        plugin_id: args.first().cloned(),
    })
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let output = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    println!("{output}");
    Ok(())
}

fn unknown_plugin_message(plugin_id: &str, records: &[PluginRuntimeRecord]) -> String {
    let mut ids = records
        .iter()
        .map(|record| record.descriptor.id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    format!(
        "unknown plugin `{plugin_id}`\navailable plugins:\n{}",
        ids.join("\n")
    )
}
