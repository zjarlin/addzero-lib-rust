use std::io::{Read, Write};
mod test {
    use az_curl::execute_curl;

    #[test]
    fn executor_returns_unauthorized_response_body() {
        let curlcmd = r#"
curl --location 'https://demo.jetlinks.cn/api/device-product/_query' \
--header 'accept: application/json, text/plain, */*' \
--header 'content-type: application/json' \
--header 'x-access-token: token-123' \
--data '{"pageIndex":0,"pageSize":96,"sorts":[{"name":"createTime","order":"desc"}],"terms":[]}'    "#;
        // let (url, join_handle) = spawn_http_server_with_status(401, response_body);

        let response = execute_curl(curlcmd).expect("401 response should still return a body");
        // let _request = join_handle.join().expect("server thread should join");
        println!("{:?}", response);
    }
}
