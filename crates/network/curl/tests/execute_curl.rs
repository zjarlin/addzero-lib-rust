use az_curl::execute::execute_curl;

#[test]
fn execute_curl_returns_response_body() {
    let command = r#"
        curl 'https://demo.jetlinks.cn/api/device-product/_query' \
          -H 'accept: application/json, text/plain, */*' \
          -H 'content-type: application/json' \
          -H 'x-access-token: token-123' \
          --data-raw '{"pageIndex":0,"pageSize":96,"sorts":[{"name":"createTime","order":"desc"}],"terms":[]}'
    "#;

    let response = execute_curl(command).expect("curl should execute");

    assert!(response.status > 0);
}
