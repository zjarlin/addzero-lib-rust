#![forbid(unsafe_code)]

use std::{env, fs, path::PathBuf};

use az_openapi_codegen::openapi_contract::{OpenApiContractConfig, generate_openai_contract};

fn main() {
    println!("cargo:rerun-if-env-changed=AZ_OPENAI_OPENAPI_SPEC_URL");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set by Cargo"));
    let generated = generate_openai_contract(OpenApiContractConfig::default())
        .expect("OpenAI OpenAPI contract generation should succeed");
    fs::write(
        out_dir.join("generated_contract.rs"),
        generated.combined_source,
    )
    .expect("generated OpenAI OpenAPI contract should be writable");
}
