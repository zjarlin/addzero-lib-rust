use crate::builder::CurlBuilder;
use crate::error::{CurlError, CurlResult};
use crate::model::ParsedCurl;
use crate::parse_support::{next_flag_value, parse_method, split_form_field, split_header};
use crate::util::normalize_command;
use reqwest::Method;

/// Parses a curl command string into a structured HTTP request.
///
/// # Errors
///
/// Returns [`CurlError`] when the command cannot be tokenized, has malformed
/// flags, or does not contain a valid URL.
pub fn parse_curl(command: impl AsRef<str>) -> CurlResult<ParsedCurl> {
    let normalized = normalize_command(command.as_ref());
    let tokens = shlex::split(&normalized).ok_or(CurlError::Tokenize)?;
    let mut iter = tokens.into_iter().peekable();

    if iter.peek().is_some_and(|token| token == "curl") {
        iter.next();
    }

    let mut builder = CurlBuilder::new(String::new());
    let mut pending_data = Vec::new();
    let mut explicit_method = None::<Method>;
    let mut saw_head = false;

    while let Some(token) = iter.next() {
        match token.as_str() {
            "-X" | "--request" => {
                let value = next_flag_value(&mut iter, "--request")?;
                explicit_method = Some(parse_method(&value)?);
            }
            "-I" | "--head" => {
                saw_head = true;
                explicit_method = Some(Method::HEAD);
            }
            "-H" | "--header" => {
                let value = next_flag_value(&mut iter, "--header")?;
                let (name, header_value) = split_header(&value)?;
                builder = builder.header(name, header_value);
            }
            "-b" | "--cookie" => {
                let value = next_flag_value(&mut iter, "--cookie")?;
                builder = builder.header("cookie", value);
            }
            "-u" | "--user" => {
                let value = next_flag_value(&mut iter, "--user")?;
                let (user, password) = value.split_once(':').unwrap_or((value.as_str(), ""));
                builder = builder.basic_auth(user, password);
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-urlencode" => {
                let value = next_flag_value(&mut iter, "--data")?;
                pending_data.push(value);
            }
            "-F" | "--form" => {
                let value = next_flag_value(&mut iter, "--form")?;
                let (name, form_value) = split_form_field(&value)?;
                builder = builder.form_field(name, form_value);
            }
            "--url" => {
                let value = next_flag_value(&mut iter, "--url")?;
                builder.url = value;
            }
            "--compressed" | "--location" | "-L" | "--silent" | "-s" | "--insecure" | "-k"
            | "--globoff" | "--verbose" | "-v" => {}
            _ if token.starts_with("http://") || token.starts_with("https://") => {
                if builder.url.is_empty() {
                    builder.url = token;
                }
            }
            _ if token.starts_with('-') => {}
            _ => {
                if builder.url.is_empty() {
                    builder.url = token;
                }
            }
        }
    }

    if !pending_data.is_empty() {
        builder = builder.body(pending_data.join("&"));
    }

    if let Some(method) = explicit_method {
        builder = builder.method(method.as_str())?;
    } else if saw_head {
        builder = builder.method(Method::HEAD.as_str())?;
    }

    builder.build()
}
