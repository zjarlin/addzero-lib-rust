use crate::error::{CurlError, CurlResult};
use reqwest::Method;

pub(crate) fn parse_method(value: &str) -> CurlResult<Method> {
    Method::from_bytes(value.trim().to_ascii_uppercase().as_bytes())
        .map_err(|_| CurlError::InvalidMethod(value.to_owned()))
}

pub(crate) fn split_header(value: &str) -> CurlResult<(String, String)> {
    let (name, body) = value
        .split_once(':')
        .ok_or_else(|| CurlError::InvalidHeader(value.to_owned()))?;
    Ok((name.trim().to_owned(), body.trim().to_owned()))
}

pub(crate) fn split_form_field(value: &str) -> CurlResult<(String, String)> {
    let (name, body) = value
        .split_once('=')
        .ok_or_else(|| CurlError::InvalidFormField(value.to_owned()))?;
    Ok((name.trim().to_owned(), body.trim().to_owned()))
}
