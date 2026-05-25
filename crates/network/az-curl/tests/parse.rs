use az_curl::parse_curl;
use reqwest::Method;

#[test]
fn parses_complex_post_command() {
    let command = r#"
        curl 'https://demo.jetlinks.cn/api/device-product/_query' \
          -H 'accept: application/json, text/plain, */*' \
          -H 'content-type: application/json' \
          -H 'x-access-token: token-123' \
          --data-raw '{"pageIndex":0,"pageSize":96,"sorts":[{"name":"createTime","order":"desc"}],"terms":[]}'
    "#;

    let parsed = parse_curl(command).expect("curl should parse");
    print!("")
}
