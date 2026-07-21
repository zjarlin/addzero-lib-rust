use az_openai_openapi::contract::{OpenAiChatApi, OpenAiResponsesApi, OpenAiVectorStoresApi};
use az_openai_openapi::models::{
    CreateChatCompletionRequest, CreateChatCompletionResponse, CreateResponse, Response,
};
use az_openai_openapi::paths::{OpenAiApiPath, OpenAiApiSpec};

#[test]
fn exposes_source_metadata_and_fixed_paths() {
    assert_eq!(OpenAiApiSpec::BASE_URL, "https://api.openai.com/v1/");
    assert_eq!(
        OpenAiApiSpec::SOURCE_REPOSITORY,
        "https://github.com/openai/openai-openapi"
    );
    assert_eq!(
        OpenAiApiSpec::SOURCE_SPEC_URL,
        "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml"
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
fn generated_sources_keep_stable_public_modules() {
    fn assert_request<T: serde::Serialize>() {}
    fn assert_response<T: for<'de> serde::Deserialize<'de>>() {}

    assert_request::<CreateChatCompletionRequest>();
    assert_response::<CreateChatCompletionResponse>();
    assert_request::<CreateResponse>();
    assert_response::<Response>();

    assert_ne!(OpenAiApiSpec::SOURCE_SPEC_VERSION, "");
    assert_ne!(OpenAiApiSpec::SOURCE_COMMIT, "");
}

#[test]
fn generated_entry_uses_checked_in_modules() {
    let generated_entry =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated.rs"))
            .expect("generated entry should be readable");

    // Keep editor diagnostics anchored to stable source files, not hash-specific target output.
    assert!(generated_entry.contains("automod::dir!(pub \"src/generated\")"));
    assert!(!generated_entry.contains("OUT_DIR"));
    assert!(!generated_entry.contains("include!"));
}
