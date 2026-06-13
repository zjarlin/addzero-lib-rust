use std::fs;
use std::path::PathBuf;

use az_openai_openapi::api::{OpenAiChatApi, OpenAiResponsesApi, OpenAiVectorStoresApi};
use az_openai_openapi::models::{
    CreateChatCompletionRequest, CreateChatCompletionResponse, CreateResponse, Response,
};
use az_openai_openapi::paths::{OpenAiApiPath, OpenAiApiSpec};

#[test]
fn exposes_source_metadata_and_fixed_paths() {
    assert_eq!(OpenAiApiSpec::BASE_URL, "https://api.openai.com/v1/");
    assert_eq!(OpenAiApiSpec::SOURCE_SPEC_VERSION, "2.3.0");
    assert_eq!(
        OpenAiApiSpec::SOURCE_COMMIT,
        "5162af98d3147432c14680df789e8e12d4891e6b"
    );
    assert_eq!(OpenAiApiPath::CHAT_BY_COMPLETIONS, "chat/completions");
    assert_eq!(OpenAiApiPath::RESPONSES, "responses");
    assert_eq!(
        OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID_BY_CONTENT,
        "vector_stores/{vector_store_id}/files/{file_id}/content"
    );
}

#[test]
fn exports_primary_api_traits() {
    fn assert_trait<T: ?Sized>() {}
    fn assert_json_dto<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}

    assert_trait::<dyn OpenAiChatApi<Error = std::io::Error>>();
    assert_trait::<dyn OpenAiResponsesApi<Error = std::io::Error>>();
    assert_trait::<dyn OpenAiVectorStoresApi<Error = std::io::Error>>();
    assert_json_dto::<CreateChatCompletionRequest>();
    assert_json_dto::<CreateChatCompletionResponse>();
    assert_json_dto::<CreateResponse>();
    assert_json_dto::<Response>();
}

#[test]
fn generated_contract_matches_observed_openapi_counts() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_dir.join("src");
    let api_dir = src_dir.join("api");
    let models_dir = src_dir.join("models");

    let operation_count = fs::read_dir(&api_dir)
        .expect("api directory should be readable")
        .map(|entry| entry.expect("api file entry should be readable").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .map(|path| {
            fs::read_to_string(path)
                .expect("api file should be readable")
                .lines()
                .filter(|line| line.trim_start().starts_with("async fn "))
                .count()
        })
        .sum::<usize>();

    let path_count = fs::read_to_string(src_dir.join("paths.rs"))
        .expect("paths.rs should be readable")
        .lines()
        .filter(|line| line.trim_start().starts_with("pub const "))
        .filter(|line| !line.contains("SOURCE_") && !line.contains("BASE_URL"))
        .count();

    let model_count = fs::read_dir(&models_dir)
        .expect("models directory should be readable")
        .map(|entry| entry.expect("model file entry should be readable").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .count();

    assert_eq!(operation_count, 242);
    assert_eq!(path_count, 162);
    assert!(model_count >= 950);
}

#[test]
fn generated_api_methods_use_typed_dtos_instead_of_global_json_fallbacks() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_dir = crate_dir.join("src/api");

    let all_api_sources = fs::read_dir(&api_dir)
        .expect("api directory should be readable")
        .map(|entry| entry.expect("api file entry should be readable").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .map(|path| fs::read_to_string(path).expect("api file should be readable"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!all_api_sources.contains("OpenAiRequestBody"));
    assert!(!all_api_sources.contains("OpenAiResponseBody"));
    assert!(all_api_sources.contains("body: CreateChatCompletionRequest"));
    assert!(all_api_sources.contains("Result<CreateChatCompletionResponse, Self::Error>"));
    assert!(all_api_sources.contains("body: CreateResponse"));
    assert!(all_api_sources.contains("Result<Response, Self::Error>"));
}
