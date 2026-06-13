use az_openai_openapi::api::{OpenAiChatApi, OpenAiResponsesApi, OpenAiVectorStoresApi};
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
fn live_macro_generation_keeps_stable_public_modules() {
    fn assert_request<T: serde::Serialize>() {}
    fn assert_response<T: for<'de> serde::Deserialize<'de>>() {}

    assert_request::<CreateChatCompletionRequest>();
    assert_response::<CreateChatCompletionResponse>();
    assert_request::<CreateResponse>();
    assert_response::<Response>();

    assert_ne!(OpenAiApiSpec::SOURCE_SPEC_VERSION, "");
    assert_ne!(OpenAiApiSpec::SOURCE_COMMIT, "");
}
