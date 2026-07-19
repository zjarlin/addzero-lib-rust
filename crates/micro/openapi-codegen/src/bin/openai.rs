#![forbid(unsafe_code)]

use std::path::PathBuf;

use az_openapi_codegen::openapi_contract::{OpenApiContractConfig, write_openai_contract};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let output_file = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_output_file);
    write_openai_contract(OpenApiContractConfig::default(), output_file)
}

fn default_output_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../api/az-openai-openapi/src")
}
