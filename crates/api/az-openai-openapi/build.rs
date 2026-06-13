#![forbid(unsafe_code)]

use std::{env, fs, path::PathBuf};

use anyhow::Context;
use az_openapi_codegen::openapi_contract::{OpenApiContractConfig, generate_openai_contract};

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-env-changed=AZ_OPENAI_OPENAPI_SPEC_URL");

    let out_dir = PathBuf::from(env::var("OUT_DIR").context("OUT_DIR should be set by Cargo")?);
    let generated = generate_openai_contract(OpenApiContractConfig::default())
        .map_err(anyhow::Error::msg)
        .context("OpenAI OpenAPI contract generation should succeed")?;
    let output_file = out_dir.join("generated_contract.rs");
    fs::write(&output_file, generated.combined_source)
        .with_context(|| format!("write generated OpenAI contract {}", output_file.display()))?;

    Ok(())
}
