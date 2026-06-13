use reqwest::Method;

pub(crate) fn next_flag_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &'static str,
) -> anyhow::Result<String> {
    iter.next()
        .ok_or_else(|| anyhow::anyhow!("flag `{flag}` requires a value"))
}

pub(crate) fn parse_method(value: &str) -> anyhow::Result<Method> {
    Method::from_bytes(value.trim().to_ascii_uppercase().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid HTTP method `{value}`"))
}

pub(crate) fn split_header(value: &str) -> anyhow::Result<(String, String)> {
    let (name, body) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("invalid header expression `{value}`"))?;
    Ok((name.trim().to_owned(), body.trim().to_owned()))
}

pub(crate) fn split_form_field(value: &str) -> anyhow::Result<(String, String)> {
    let (name, body) = value
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("invalid form expression `{value}`"))?;
    Ok((name.trim().to_owned(), body.trim().to_owned()))
}
