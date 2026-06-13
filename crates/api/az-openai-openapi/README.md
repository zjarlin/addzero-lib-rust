# az-openai-openapi

OpenAI REST OpenAPI contract traits, typed DTOs, and fixed REST path constants.

## Source

- OpenAPI repository: https://github.com/openai/openai-openapi
- Spec file: https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml
- Source spec version observed during generation: `2.3.0`
- Source commit observed during generation: `5162af98d3147432c14680df789e8e12d4891e6b`

This crate is implementation-free. Application crates implement the generated traits and decide how to map requests to HTTP clients, auth, retries, and streaming.

The generated traits use concrete `Serialize`/`Deserialize` DTOs for JSON request and response bodies. For example, `OpenAiChatApi::create_chat_completion` accepts `CreateChatCompletionRequest` and returns `CreateChatCompletionResponse`, while `OpenAiResponsesApi::create_response` accepts `CreateResponse` and returns `Response`.

`OpenAiJsonValue` and `OpenAiJsonObject` are retained only for fields whose OpenAPI schema is intentionally open-ended or lacks a fixed object shape. They are not used as global request/response fallbacks in the REST trait methods.

## Regeneration

```bash
python3 crates/api/az-openai-openapi/scripts/generate-openai-api.py --crate-dir crates/api/az-openai-openapi
cargo fmt -p az-openai-openapi
cargo test -p az-openai-openapi
```

Module collection follows the addzero automod convention: `src/lib.rs`, `src/api.rs`, and `src/models.rs` stay as entrypoints using `automod::dir!`.
