use addzero_music::*;
use reqwest::header::ACCEPT;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
fn facade_builds_default_clients() {
    let _ = Music::netease().expect("default netease client should build");
    let _ = Music::suno("token-123").expect("default suno client should build");
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

fn test_music_config(base_url: &str) -> MusicResult<ApiConfig> {
    ApiConfig::builder(base_url)
        .default_header("Referer", "https://music.163.com/")
        .user_agent("Mozilla/5.0")
        .build()
}

fn test_suno_config(base_url: &str) -> MusicResult<ApiConfig> {
    ApiConfig::builder(base_url)
        .default_header(ACCEPT.as_str(), "application/json")
        .build()
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
    let request_line = lines
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line"))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_owned();
    let path = request_parts.next().unwrap_or_default().to_owned();

    let mut headers = BTreeMap::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            let normalized_name = name.trim().to_ascii_lowercase();
            let normalized_value = value.trim().to_owned();
            if normalized_name == "content-length" {
                content_length = normalized_value.parse::<usize>().unwrap_or(0);
            }
            headers.insert(normalized_name, normalized_value);
        }
    }

    let mut body = buffer[header_end..].to_vec();
    while body.len() < content_length {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }

    Ok(CapturedRequest {
        method,
        path,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
    let status_text = match response.status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let payload = response.body.as_bytes();
    let response_text = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        status_text,
        response.content_type,
        payload.len()
    );
    stream.write_all(response_text.as_bytes())?;
    stream.write_all(payload)?;
    stream.flush()
}
