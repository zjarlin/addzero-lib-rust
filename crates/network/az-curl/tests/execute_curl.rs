use az_curl::execute_curl;

#[test]
fn execute_curl_returns_unauthorized_response_body() {
    let command = r#"
    curl --location 'https://demo.jetlinks.cn/api/device-product/_query' \
--header 'accept: application/json, text/plain, */*' \
--header 'content-type: application/json' \
--header 'x-access-token: token-123' \
--data '{"pageIndex":0,"pageSize":96,"sorts":[{"name":"createTime","order":"desc"}],"terms":[]}'"#;



    let response = execute_curl(command).expect("401 response should still return a body");


    println!("{:?}", response.text_lossy());

}
