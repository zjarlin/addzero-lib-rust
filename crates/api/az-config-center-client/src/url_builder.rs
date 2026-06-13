use reqwest::Url;

pub(crate) fn build_url(base_url: &Url, path: &str, query: &[(&str, String)]) -> Url {
    let mut url = base_url.clone();
    url.set_path(path);
    url.set_query(None);
    if !query.is_empty() {
        let mut pairs = url.query_pairs_mut();
        for (name, value) in query {
            pairs.append_pair(name, value);
        }
    }
    url
}

#[cfg(test)]
mod tests {
    use reqwest::Url;

    use super::build_url;

    #[test]
    fn build_url_encodes_query_pairs() {
        let base_url = Url::parse("http://127.0.0.1:8080").expect("base url should parse");
        let url = build_url(
            &base_url,
            "/api/v1/config/value",
            &[
                ("namespace", "cmp aio.dev".to_owned()),
                ("key", "redis.host".to_owned()),
            ],
        );

        // 关键断言：查询参数交给 URL 类型编码，配置键和命名空间可安全包含空格等字符。
        assert_eq!(
            url.as_str(),
            "http://127.0.0.1:8080/api/v1/config/value?namespace=cmp+aio.dev&key=redis.host"
        );
    }
}
