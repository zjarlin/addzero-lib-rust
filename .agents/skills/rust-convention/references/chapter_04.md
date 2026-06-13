# Chapter 4 - Errors Handling

Rust enforces a strict error handling approach, but *how* you handle them defines where your code feels ergonomic, consistent and safe - as opposing cryptic and painful. This chapter dives into best practices for modeling and managing fallible operations across libraries and binaries.

> Even if you decide to crash you application with `unwrap` or `expect`, Rust forces you to declare that intentionally.

## 4.1 Prefer `Result`, avoid panic 🫨

Rust has a powerful type that wraps fallible data, [`Result<T, E>`](https://doc.rust-lang.org/std/result/), this allows us to handle Error cases according to our needs and manage the state of the application based on that.

* If your function can fail, prefer to return a `Result`:
```rust
fn divide(x: f64, y: f64) -> Result<f64, DivisionError> {
    if y == 0.0 {
        Err(DivisionError::DividedByZero)
    } else {
        Ok(x / y)
    }
}
```

* Use `panic!` only in unrecoverable conditions - typically tests, assertions, bugs or a need to crash the application for some explicit reason.
* There are 3 relevant macros that can replace `panic!` in appropriate conditions:
    * `todo!`, similar to panic, but alerts the compiler that you are aware that there is code missing.
    * `unreachable!`, you have reasoned about the code block and are sure that condition `xyz` is not possible and if ever becomes possible you want to be alerted.
    * `unimplemented!`, specially useful for alerting that a block is not yet implement with a reason.

## 4.2 Avoid `unwrap`/`expect` in Production

Although `expect` is preferred to `unwrap`, as it can have context, they should be avoided in production code as there are smarter alternatives to them. Considering that, they should be used in the following scenarios:
- In tests, assertions or test helper functions.
- When failure is impossible.
- When the smarter options can't handle the specific case.

### 🚨 Alternative ways of handling `unwrap`/`expect`:

* If your `Result` (or `Option`) can have a predefined early return value in case of `Result::Err`, that doesn't need to know the `Err` value, use `let Ok(..) = else { return ... }` pattern, as it helps with flatten functions:
```rust
let Ok(json) = serde_json::from_str(&input) else {
    return Err(MyError::InvalidJson);
}
```
* If your `Result` (or `Option`) needs error recovery in case of `Result::Err`, that doesn't need to know the `Err` value, use `if let Ok(..) else { ... }` pattern:
```rust
if let Ok(json) = serde_json::from_str(&input) else {
    ...
} else {
    Err(do_something_with_input(&input))
}
```
* Functions that can have to handle `Option::None` values are recommended to return `Result<T, E>`, where `E` is a crate or module level error, like the examples above.
* Lastly `unwrap_or`, `unwrap_or_else` or `unwrap_or_default`, these functions help you create alternative exits to unwrap that manage the uninitialized values.

## 4.3 Repository convention: use `anyhow` for crate-level errors

This repository intentionally standardizes on `anyhow::Result<T>` for fallible APIs. Do not create a crate-local `XxxError` enum and `XxxResult<T>` alias merely to wrap I/O, JSON, model loading, parsing, or validation failures.

```rust
use anyhow::{Context, Result, bail};

pub fn load_config(path: &std::path::Path) -> Result<Config> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config `{}`", path.display()))?;
    let config = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse config `{}`", path.display()))?;

    if config.name.trim().is_empty() {
        bail!("config `{}` has blank name", path.display());
    }

    Ok(config)
}
```

Keep protocol/data model error structs or enums when they are part of a wire contract, API payload, or user-facing response schema. Remove Rust-only error plumbing when `anyhow::Result` can carry the same failure with context.

## 4.4 Use context with `anyhow`

`anyhow` is useful only when errors keep their source chain and carry enough operation-specific context:

```rust
use anyhow::{Context, Result, anyhow};

fn main() -> Result<()> {
    let content = std::fs::read_to_string("config.json")
        .context("Failed to read config file")?;
    Config::from_str(&content)
        .map_err(|err| anyhow!("Config parsing error: {err}"))
}
```

### Anyhow gotchas

* Do not replace source errors with fixed strings.
* Prefer `?` for direct source propagation and `.with_context(...)` when the operation target matters.
* Use `bail!` for validation failures that have no source error.

## 4.5 Use `?` to Bubble Errors

Prefer using `?` over verbose alternatives like `match` chains:
```rust
fn handle_request(req: &Request) -> Result<ValidatedRequest, MyError> {
    validate_headers(req)?;
    validate_body_format(req)?;
    validate_credentials(req)?;
    let body = Body::try_from(req)?;

    Ok(ValidatedRequest::try_from((req, body))?)
}
```

> In case error recovery is needed, use `or_else`, `map_err`, `if let Ok(..) else`. To **inspect or log your error**, use `inspect_err`.

### Split complex error construction

When returning a domain error that wraps converted paths, generated source errors, parsed metadata, or multiple constructor arguments, split the pieces into local variables first. This keeps the error boundary readable, debuggable, and easy to extend with logging or context.

❌ Avoid nesting every step inside one `return Err(...)`:

```rust
return Err(OnnxImageError::io(
    path.to_path_buf(),
    std::io::Error::new(std::io::ErrorKind::NotFound, "ONNX model file not found"),
));
```

✅ Prefer naming each part before wrapping it:

```rust
let model_path = path.to_path_buf();
let source = std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "ONNX model file not found",
);
let error = OnnxImageError::io(model_path, source);

return Err(error);
```

This is required when an error expression combines three or more concerns, such as data conversion, source error creation, and domain error mapping. Tiny single-constructor returns like `return Err(MyError::MissingConfig);` are still fine.

When the function returns `anyhow::Result`, prefer `anyhow::bail!(...)` for validation failures instead of constructing `anyhow!(...)` inside `return Err(...)`:

```rust
anyhow::bail!("ONNX model file not found: `{}`", path.display());
```

## 4.6 Unit Test should exercise errors

While many errors don't implement PartialEq and Eq, making it hard to do direct assertions between them, it is possible to check the error messages with `format!` or `to_string()`, making the errors meaningful and test validated:

```rust
#[test]
fn error_does_not_implement_partial_eq() {
    let err = divide(10., 0.0).unwrap_err();
    assert_eq!(err.to_string(), "division by zero");
}

#[test]
fn error_implements_partial_eq() {
    let err = process(my_value).unwrap_err();

    assert_eq!(
        err,
        MyError {
            ..
        }
    )
}
```

## 4.7 Important Topics

### Custom Error Structs

Sometimes you don't need an enum to handle your errors, as there is only one type of error that your module can have. This can be solved with `struct Errors`:

```rust
#[derive(Debug, thiserror::Error, PartialEq)]
#[error("Request failed with code `{code}`: {message}")]
struct HttpError {
    code: u16,
    message: String
}
```

### Async Errors

When using async runtimes, like Tokio, make sure that your errors implement `Send + Sync + 'static` where needed, specially in tasks or across `.await` boundaries:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ...
    Ok(())
}
```

> Avoid `Box<dyn std::error::Error>` in libraries unless it is really needed
