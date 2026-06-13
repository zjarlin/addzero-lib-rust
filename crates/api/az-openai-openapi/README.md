# az-openai-openapi

OpenAI REST OpenAPI contract traits and fixed REST path constants.

## Source

- OpenAPI repository: https://github.com/openai/openai-openapi
- Spec file: https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml
- Source spec version observed during generation: `2.3.0`
- Source commit observed during generation: `5162af98d3147432c14680df789e8e12d4891e6b`

This crate is implementation-free. Application crates implement the generated traits and decide how to map requests to HTTP clients, auth, retries, streaming, and typed domain models.
