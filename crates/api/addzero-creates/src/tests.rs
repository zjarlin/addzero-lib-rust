use crate::*;
use reqwest::Url;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HOST};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[test]
fn maven_search_parses_latest_version() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![TestResponse::json(
        r#"{"response":{"docs":[{"id":"com.google.guava:guava","g":"com.google.guava","a":"guava","latestVersion":"33.2.1-jre","p":"bundle","timestamp":123456}]}}"#,
    )])?;

    let api = MavenCentralApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let artifacts = api.search_by_coordinates("com.google.guava", "guava", 5)?;

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].group_id, "com.google.guava");
    assert_eq!(artifacts[0].artifact_id, "guava");
    assert_eq!(artifacts[0].resolved_version(), Some("33.2.1-jre"));

    let requests = server.finish()?;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert!(
        requests[0]
            .path
            .contains("/solrsearch/select?q=g%3Acom.google.guava+AND+a%3Aguava")
    );
    assert!(requests[0].path.contains("rows=5"));
    Ok(())
}

#[test]
fn maven_download_uses_remotecontent_endpoint() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![TestResponse::text("artifact-body")])?;

    let api = MavenCentralApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let bytes = api.download_file(
        "com.google.guava",
        "guava",
        "33.2.1-jre",
        "guava-33.2.1-jre.pom",
    )?;

    assert_eq!(String::from_utf8(bytes)?, "artifact-body");

    let requests = server.finish()?;
    assert_eq!(
        requests[0].path,
        "/remotecontent?filepath=com%2Fgoogle%2Fguava%2Fguava%2F33.2.1-jre%2Fguava-33.2.1-jre.pom"
    );
    Ok(())
}

#[test]
fn temp_mail_create_mailbox_and_login_runs_full_flow() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"hydra:member":[{"id":"domain-1","domain":"mail.tm","isActive":true,"isPrivate":false}]}"#,
        ),
        TestResponse::json(r#"{"id":"account-1"}"#),
        TestResponse::json(r#"{"token":"token-1"}"#),
    ])?;

    let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let mailbox = api.create_mailbox_and_login("az_", 10)?;

    assert!(mailbox.address.ends_with("@mail.tm"));
    assert_eq!(mailbox.account_id, "account-1");
    assert_eq!(mailbox.token, "token-1");
    assert_eq!(mailbox.password.len(), 10);

    let requests = server.finish()?;
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].path, "/domains");
    assert_eq!(requests[1].path, "/accounts");
    assert_eq!(requests[2].path, "/token");
    assert!(requests[1].body.contains("\"address\""));
    assert!(requests[2].body.contains("\"password\""));
    Ok(())
}

#[test]
fn temp_mail_get_message_flattens_html_array() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![TestResponse::json(
        r#"{"id":"msg-1","from":{"address":"from@mail.tm","name":"Sender"},"to":[{"address":"to@mail.tm","name":"Receiver"}],"subject":"Hello","text":"Plain","html":["<p>Hello</p>"],"createdAt":"2026-04-20T12:00:00.000Z"}"#,
    )])?;

    let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let message = api.get_message("token-1", "msg-1")?;

    assert_eq!(message.id, "msg-1");
    assert_eq!(message.html, "<p>Hello</p>");
    assert_eq!(message.to.len(), 1);

    let requests = server.finish()?;
    let authorization = requests[0]
        .headers
        .get("authorization")
        .cloned()
        .ok_or_else(|| std::io::Error::other("missing authorization header"))?;
    assert_eq!(requests[0].path, "/messages/msg-1");
    assert_eq!(authorization, "Bearer token-1");
    Ok(())
}

#[test]
fn music_search_supports_song_artist_album_and_playlist_queries() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"code":200,"result":{"songs":[{"id":1,"name":"晴天","artists":[{"id":11,"name":"周杰伦"}],"album":{"id":21,"name":"叶惠美"},"duration":269000}]}}"#,
        ),
        TestResponse::json(
            r#"{"code":200,"result":{"artists":[{"id":11,"name":"周杰伦","albumSize":15,"musicSize":200}]}}"#,
        ),
        TestResponse::json(
            r#"{"code":200,"result":{"albums":[{"id":21,"name":"叶惠美","artist":{"id":11,"name":"周杰伦"}}]}}"#,
        ),
        TestResponse::json(
            r#"{"code":200,"result":{"playlists":[{"id":31,"name":"华语经典","trackCount":128,"playCount":9999}]}}"#,
        ),
    ])?;

    let api = MusicSearchApi::new(test_music_config(server.base_url())?)?;
    let songs = api.search_songs("晴天", 5, 0)?;
    let artists = api.search_artists("周杰伦", 3, 0)?;
    let albums = api.search_albums("叶惠美", 3, 0)?;
    let playlists = api.search_playlists("华语经典", 3, 0)?;

    assert_eq!(songs[0].name, "晴天");
    assert_eq!(songs[0].artists[0].name, "周杰伦");
    assert_eq!(artists[0].name, "周杰伦");
    assert_eq!(albums[0].name, "叶惠美");
    assert_eq!(playlists[0].name, "华语经典");

    let requests = server.finish()?;
    assert_eq!(requests.len(), 4);
    assert!(
        requests[0]
            .path
            .contains("/search/get/web?s=%E6%99%B4%E5%A4%A9")
    );
    assert!(requests[0].path.contains("type=1"));
    assert!(requests[1].path.contains("type=100"));
    assert!(requests[2].path.contains("type=10"));
    assert!(requests[3].path.contains("type=1000"));
    assert_eq!(
        requests[0].headers.get("referer").map(String::as_str),
        Some("https://music.163.com/")
    );
    Ok(())
}

#[test]
fn music_lyric_detail_and_filtering_behave_like_jvm_client() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(r#"{"code":200,"lrc":{"version":1,"lyric":"[00:01]晴天"}}"#),
        TestResponse::json(
            r#"{"code":200,"songs":[{"id":1,"name":"晴天","artists":[{"id":11,"name":"周杰伦"}],"album":{"id":21,"name":"叶惠美"}},{"id":2,"name":"七里香","artists":[{"id":11,"name":"周杰伦"}],"album":{"id":22,"name":"七里香"}}]}"#,
        ),
        TestResponse::json(
            r#"{"code":200,"result":{"songs":[{"id":1,"name":"晴天","artists":[{"id":11,"name":"周杰伦"}]},{"id":2,"name":"晴天","artists":[{"id":12,"name":"别人"}]}]}}"#,
        ),
    ])?;

    let api = MusicSearchApi::new(test_music_config(server.base_url())?)?;
    let lyric = api.get_lyric(1)?;
    let songs = api.get_song_detail(&[1, 2])?;
    let filtered = api.search_by_song_and_artist("晴天", Some("周杰伦"))?;

    assert_eq!(
        lyric.lrc.and_then(|item| item.lyric).as_deref(),
        Some("[00:01]晴天")
    );
    assert_eq!(songs.len(), 2);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].artists[0].name, "周杰伦");

    let requests = server.finish()?;
    assert_eq!(requests[0].path, "/song/lyric?id=1&lv=1&tv=1");
    assert_eq!(requests[1].path, "/song/detail?ids=%5B1%2C2%5D");
    assert!(
        requests[2]
            .path
            .contains("/search/get/web?s=%E6%99%B4%E5%A4%A9+%E5%91%A8%E6%9D%B0%E4%BC%A6")
    );
    Ok(())
}

#[test]
fn suno_endpoints_use_bearer_token_and_decode_payloads() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(r#"{"code":200,"message":"ok","data":"task-1"}"#),
        TestResponse::json(r#"{"code":200,"message":"ok","data":"歌词"}"#),
        TestResponse::json(r#"{"code":200,"message":"ok","data":"concat-1"}"#),
        TestResponse::json(
            r#"{"code":200,"message":"ok","data":{"id":"task-1","status":"complete","audio_url":"https://example.com/1.mp3"}}"#,
        ),
        TestResponse::json(
            r#"{"code":200,"message":"ok","data":[{"id":"task-1","status":"complete"},{"id":"task-2","status":"streaming"}]}"#,
        ),
    ])?;

    let api = SunoApi::new("token-123", test_suno_config(server.base_url())?)?;
    let task_id = api.generate_music(&SunoMusicRequest {
        prompt: "写一首歌".to_owned(),
        title: Some("测试".to_owned()),
        tags: Some("pop".to_owned()),
        ..Default::default()
    })?;
    let lyrics = api.generate_lyrics("写一段歌词")?;
    let concat_task = api.concat_songs("clip-1")?;
    let task = api.fetch_task("task-1")?;
    let tasks = api.batch_fetch_tasks(vec!["task-1", "task-2"])?;

    assert_eq!(task_id, "task-1");
    assert_eq!(lyrics, "歌词");
    assert_eq!(concat_task, "concat-1");
    assert_eq!(
        task.and_then(|item| item.audio_url).as_deref(),
        Some("https://example.com/1.mp3")
    );
    assert_eq!(tasks.len(), 2);

    let requests = server.finish()?;
    assert_eq!(requests.len(), 5);
    for request in &requests {
        assert_eq!(
            request.headers.get("authorization").map(String::as_str),
            Some("Bearer token-123")
        );
    }
    assert_eq!(requests[0].path, "/suno/submit/music");
    assert!(requests[0].body.contains("\"prompt\":\"写一首歌\""));
    assert_eq!(requests[1].path, "/suno/lyrics");
    assert_eq!(requests[2].path, "/suno/concat");
    assert_eq!(requests[3].path, "/suno/fetch/task-1");
    assert_eq!(requests[4].path, "/suno/fetch");
    Ok(())
}

#[test]
fn suno_wait_for_completion_polls_until_complete() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"code":200,"message":"ok","data":{"id":"task-1","status":"processing"}}"#,
        ),
        TestResponse::json(
            r#"{"code":200,"message":"ok","data":{"id":"task-1","status":"complete","audio_url":"https://example.com/done.mp3"}}"#,
        ),
    ])?;

    let api = SunoApi::new("token-123", test_suno_config(server.base_url())?)?;
    let mut seen_status = Vec::new();
    let task = api.wait_for_completion_with(
        "task-1",
        Duration::from_millis(50),
        Duration::from_millis(1),
        |status| seen_status.push(status.map(ToOwned::to_owned)),
    )?;

    assert_eq!(task.status.as_deref(), Some("complete"));
    assert_eq!(
        seen_status,
        vec![Some("processing".to_owned()), Some("complete".to_owned())]
    );
    Ok(())
}

#[test]
fn tianyancha_search_and_base_info_send_required_headers() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"state":"ok","message":"success","data":{"companyList":[{"id":3398690435,"name":"河南中洛佳科技有限公司","alias":"中洛佳","legalPersonName":"马丽北","regStatus":"存续"}],"companyTotal":1,"companyTotalPage":1,"searchContent":"中洛佳"}}"#,
        ),
        TestResponse::json(
            r#"{"state":"ok","message":"success","data":{"id":3398690435,"name":"河南中洛佳科技有限公司","alias":"中洛佳","legalPersonName":"马丽北","businessScope":"软件开发","phoneList":["0379-65199909"],"emailList":["demo@example.com"]}}"#,
        ),
    ])?;

    let api = TianyanchaApi::new(
        "auth-value",
        "token-value",
        test_tianyancha_config(server.base_url())?,
    )?;
    let search = api.search_company("中洛佳", 1, 10, "0")?;
    let detail = api.get_base_info(3398690435)?;

    assert_eq!(search.company_list.len(), 1);
    assert_eq!(detail.name, "河南中洛佳科技有限公司");

    let requests = server.finish()?;
    assert!(
        requests[0]
            .path
            .contains("/services/v3/search/sNorV4/%E4%B8%AD%E6%B4%9B%E4%BD%B3")
    );
    assert!(requests[0].path.contains("pageNum=1"));
    assert_eq!(
        requests[0].headers.get("authorization").map(String::as_str),
        Some("auth-value")
    );
    assert_eq!(
        requests[0].headers.get("x-auth-token").map(String::as_str),
        Some("token-value")
    );
    assert_eq!(
        requests[0].headers.get("version").map(String::as_str),
        Some("TYC-XCX-WX")
    );
    assert_eq!(
        requests[1].path,
        "/services/v3/t/common/baseinfoV5/3398690435"
    );
    Ok(())
}

#[test]
fn huawei_tianyancha_search_generates_signature_headers() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![TestResponse::json(
        r#"{"code":200,"msg":"成功","success":true,"data":{"companyList":[{"companyCode":"410307100005658","companyName":"洛阳古城机械有限公司","companyStatus":"在营（开业）","creditNo":"91410307171359173W","establishDate":"20030415","legalPerson":"王根成"}],"orderNo":"202511032218555320095","pageInfo":{"pageIndex":"2","pageSize":"20","totalRecords":"528"}}}"#,
    )])?;

    let api = TianyanchaHuaweiApi::new(
        "ak-demo",
        "sk-demo",
        ApiConfig::builder(server.base_url()).build()?,
    )?;
    let data = api.search_companies("测试企业", 2, 20)?;

    assert_eq!(data.company_list.len(), 1);
    assert_eq!(
        data.page_info.as_ref().map(|item| item.page_index.as_str()),
        Some("2")
    );

    let requests = server.finish()?;
    assert_eq!(
        requests[0].path,
        "/api-mall/api/company_search/query?keyword=%E6%B5%8B%E8%AF%95%E4%BC%81%E4%B8%9A&pageNum=2&pageSize=20"
    );
    let authorization = requests[0]
        .headers
        .get("authorization")
        .cloned()
        .ok_or_else(|| std::io::Error::other("missing authorization header"))?;
    assert!(authorization.starts_with("SDK-HMAC-SHA256 Access=ak-demo"));
    assert!(authorization.contains("SignedHeaders=host;x-sdk-date"));
    assert!(requests[0].headers.contains_key("x-sdk-date"));
    Ok(())
}

#[test]
fn huawei_signature_canonicalizes_query_string() -> Result<(), Box<dyn Error>> {
    let api = TianyanchaHuaweiApi::new(
        "ak-demo",
        "sk-demo",
        ApiConfig::builder("http://example.com").build()?,
    )?;
    let url = Url::parse(
        "http://example.com/api-mall/api/company_search/query?pageSize=20&keyword=%E6%B5%8B%E8%AF%95&pageNum=2",
    )?;
    let headers = api.sign_headers("GET", &url, None, Some("20260421T120000Z"))?;

    assert_eq!(
        crate::util::canonical_query_string(&url),
        "keyword=%E6%B5%8B%E8%AF%95&pageNum=2&pageSize=20"
    );
    assert_eq!(
        headers.get("X-Sdk-Date").map(String::as_str),
        Some("20260421T120000Z")
    );
    assert!(
        headers
            .get("authorization")
            .or_else(|| headers.get("Authorization"))
            .is_some()
    );
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedRequest {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: String,
}

#[derive(Debug, Clone)]
struct TestResponse {
    status: u16,
    content_type: &'static str,
    body: String,
}

impl TestResponse {
    fn json(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "application/json",
            body: body.to_owned(),
        }
    }

    fn text(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "text/plain; charset=utf-8",
            body: body.to_owned(),
        }
    }
}

fn test_music_config(base_url: &str) -> CreatesResult<ApiConfig> {
    ApiConfig::builder(base_url)
        .default_header("Referer", "https://music.163.com/")
        .user_agent("Mozilla/5.0")
        .build()
}

fn test_suno_config(base_url: &str) -> CreatesResult<ApiConfig> {
    ApiConfig::builder(base_url)
        .default_header(ACCEPT.as_str(), "application/json")
        .build()
}

fn test_tianyancha_config(base_url: &str) -> CreatesResult<ApiConfig> {
    ApiConfig::builder(base_url)
        .default_header(CONTENT_TYPE.as_str(), "application/json")
        .default_header(HOST.as_str(), "api9.tianyancha.com")
        .default_header(ACCEPT.as_str(), "*/*")
        .default_header("version", "TYC-XCX-WX")
        .default_header("User-Agent", "Mozilla/5.0")
        .default_header("Accept-Language", "zh-cn")
        .build()
}

struct TestServer {
    base_url: String,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
    handle: Option<JoinHandle<std::io::Result<()>>>,
}

impl TestServer {
    fn spawn(responses: Vec<TestResponse>) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = Arc::clone(&captured);

        let handle = thread::spawn(move || -> std::io::Result<()> {
            for response in responses {
                let (mut stream, _) = listener.accept()?;
                let request = read_request(&mut stream)?;
                let mut guard = captured_clone
                    .lock()
                    .map_err(|_| std::io::Error::other("request capture mutex poisoned"))?;
                guard.push(request);
                drop(guard);
                write_response(&mut stream, response)?;
            }
            Ok(())
        });

        Ok(Self {
            base_url: format!("http://{address}"),
            captured,
            handle: Some(handle),
        })
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn finish(mut self) -> Result<Vec<CapturedRequest>, Box<dyn Error>> {
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => result?,
                Err(_) => {
                    return Err(Box::new(std::io::Error::other(
                        "test server thread panicked",
                    )));
                }
            }
        }

        let guard = self
            .captured
            .lock()
            .map_err(|_| std::io::Error::other("request capture mutex poisoned"))?;
        Ok(guard.clone())
    }
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<CapturedRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "request ended before headers",
            ));
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(index) = find_bytes(&buffer, b"\r\n\r\n") {
            break index + 4;
        }
    };

    let header_text = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line")
    })?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing method"))?
        .to_owned();
    let path = request_parts
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing path"))?
        .to_owned();

    let mut headers = BTreeMap::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }

        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let normalized_name = name.trim().to_ascii_lowercase();
        let trimmed_value = value.trim().to_owned();
        if normalized_name == "content-length" {
            content_length = trimmed_value.parse::<usize>().unwrap_or(0);
        }
        headers.insert(normalized_name, trimmed_value);
    }

    while buffer.len() < header_end + content_length {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }

    let body_bytes = if content_length == 0 {
        &[][..]
    } else {
        &buffer[header_end..header_end + content_length]
    };

    Ok(CapturedRequest {
        method,
        path,
        headers,
        body: String::from_utf8_lossy(body_bytes).into_owned(),
    })
}

fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
    let body = response.body;
    let payload = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status,
        response.content_type,
        body.len(),
        body
    );
    stream.write_all(payload.as_bytes())?;
    stream.flush()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
