# az-openai-openapi

OpenAI REST OpenAPI contract traits, typed DTOs, and fixed REST path constants.

## Source

- OpenAPI repository: https://github.com/openai/openai-openapi
- Spec file: https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml
- Source metadata is captured by the compile-time Rust macro in `OpenAiApiSpec`.

This crate is implementation-free. Application crates implement the generated traits and decide how to map requests to HTTP clients, auth, retries, and streaming.

The generated traits use concrete `Serialize`/`Deserialize` DTOs for JSON request and response bodies. For example, `OpenAiChatApi::create_chat_completion` accepts `CreateChatCompletionRequest` and returns `CreateChatCompletionResponse`, while `OpenAiResponsesApi::create_response` accepts `CreateResponse` and returns `Response`.

`OpenAiJsonValue` and `OpenAiJsonObject` are retained only for fields whose OpenAPI schema is intentionally open-ended or lacks a fixed object shape. They are not used as global request/response fallbacks in the REST trait methods.

## Generation

The contract is generated at compile time by `az-openapi-macro` in `crates/micro`.
`src/lib.rs` stays as an automod entrypoint, and `src/openai_contract.rs` invokes the macro.

The macro fetches the remote OpenAPI spec during compilation. The default source is the official `openapi.yaml` because `openai/openai-openapi` does not currently publish `openapi.json` at the matching raw GitHub URL. To generate from a remote JSON spec mirror, set `AZ_OPENAI_OPENAPI_SPEC_URL`:

```bash
AZ_OPENAI_OPENAPI_SPEC_URL=https://example.com/openapi.json cargo check -p az-openai-openapi
```

Run the verification with:

```bash
cargo fmt -p az-openapi-macro -p az-openai-openapi
cargo test -p az-openai-openapi
```

Module collection follows the addzero automod convention: `src/lib.rs` stays as the crate entrypoint using `automod::dir!`, while `src/api.rs`, `src/models.rs`, `src/paths.rs`, and `src/bodies.rs` keep stable public module paths by re-exporting the macro-generated contract.
