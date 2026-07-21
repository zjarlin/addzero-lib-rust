use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::route_table::RouteTable;

const MAX_REQUEST_HEAD_SIZE: usize = 64 * 1024;

/// 启动按 HTTP Host 头分流的公网 relay。
pub async fn serve(listen: SocketAddr, routes_path: PathBuf) -> Result<()> {
    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("relay 监听失败：{listen}"))?;
    let routes_path = Arc::new(routes_path);
    println!("addhost relay 正在监听 http://{listen}");

    loop {
        let (stream, peer) = listener.accept().await.context("relay 接受连接失败")?;
        let routes_path = Arc::clone(&routes_path);
        tokio::spawn(async move {
            if let Err(error) = proxy_connection(stream, routes_path.as_ref()).await {
                eprintln!("relay 连接 {peer} 失败：{error:#}");
            }
        });
    }
}

async fn proxy_connection(mut incoming: TcpStream, routes_path: &Path) -> Result<()> {
    let request_head = read_request_head(&mut incoming).await?;
    let host = match parse_host(&request_head) {
        Ok(host) => host,
        Err(error) => {
            write_error(&mut incoming, 400, "Bad Request").await?;
            return Err(error);
        }
    };
    let routes = RouteTable::load(routes_path)?;
    let Some(remote_port) = routes.resolve(&host) else {
        write_error(&mut incoming, 404, "Not Found").await?;
        return Ok(());
    };

    let mut upstream = match TcpStream::connect(("127.0.0.1", remote_port)).await {
        Ok(stream) => stream,
        Err(error) => {
            write_error(&mut incoming, 502, "Bad Gateway").await?;
            return Err(error)
                .with_context(|| format!("连接 SSH 回环端口 127.0.0.1:{remote_port} 失败"));
        }
    };
    upstream
        .write_all(&request_head)
        .await
        .context("转发首个 HTTP 请求失败")?;
    tokio::io::copy_bidirectional(&mut incoming, &mut upstream)
        .await
        .context("双向转发 HTTP 连接失败")?;
    Ok(())
}

async fn read_request_head(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut request = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];

    loop {
        let read_count = stream
            .read(&mut chunk)
            .await
            .context("读取 HTTP 请求失败")?;
        if read_count == 0 {
            bail!("客户端在发送完整 HTTP 请求头之前断开");
        }
        request.extend_from_slice(&chunk[..read_count]);

        if request.len() > MAX_REQUEST_HEAD_SIZE {
            write_error(stream, 431, "Request Header Fields Too Large").await?;
            bail!("HTTP 请求头超过 {MAX_REQUEST_HEAD_SIZE} 字节");
        }
        if find_header_end(&request).is_some() {
            return Ok(request);
        }
    }
}

fn parse_host(request: &[u8]) -> Result<String> {
    let header_end = find_header_end(request).context("HTTP 请求头不完整")?;
    let header_text = String::from_utf8_lossy(&request[..header_end]);

    for line in header_text.lines().skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("host") {
            let host = strip_host_port(value.trim());
            if host.is_empty() {
                bail!("HTTP Host 头为空");
            }
            return Ok(host.to_ascii_lowercase());
        }
    }
    bail!("HTTP 请求缺少 Host 头")
}

fn strip_host_port(value: &str) -> &str {
    if value.starts_with('[') {
        return value
            .split_once(']')
            .map_or(value, |(host, _)| host.trim_start_matches('['));
    }
    value.split_once(':').map_or(value, |(host, _)| host)
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

async fn write_error(stream: &mut TcpStream, status: u16, reason: &str) -> Result<()> {
    let body = format!("{status} {reason}\n");
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nConnection: close\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .await
        .context("写入 relay 错误响应失败")
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use tempfile::tempdir;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    };

    use super::*;
    use crate::route_table::RouteTable;

    #[test]
    fn parses_host_header_and_port() -> Result<()> {
        let request = b"GET / HTTP/1.1\r\nHost: Demo.Example.com:80\r\n\r\n";
        assert_eq!(parse_host(request)?, "demo.example.com");
        Ok(())
    }

    #[test]
    fn rejects_missing_host_header() {
        let request = b"GET / HTTP/1.1\r\nUser-Agent: test\r\n\r\n";
        assert!(parse_host(request).is_err());
    }

    #[tokio::test]
    async fn proxies_http_connection_by_host() -> Result<()> {
        let upstream_listener = TcpListener::bind("127.0.0.1:0").await?;
        let upstream_port = upstream_listener.local_addr()?.port();
        let upstream_task = tokio::spawn(async move {
            let (mut stream, _) = upstream_listener.accept().await?;
            let mut request = [0_u8; 1024];
            let read_count = stream.read(&mut request).await?;
            let request = String::from_utf8_lossy(&request[..read_count]);
            if !request.contains("Host: demo.example.com") {
                bail!("上游没有收到原始 Host 头");
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 2\r\n\r\nOK")
                .await?;
            Result::<()>::Ok(())
        });

        let directory = tempdir()?;
        let routes_path = directory.path().join("routes.toml");
        let mut routes = RouteTable::default();
        routes.set("demo.example.com", upstream_port)?;
        routes.save(&routes_path)?;

        let relay_listener = TcpListener::bind("127.0.0.1:0").await?;
        let relay_address = relay_listener.local_addr()?;
        let relay_routes_path = routes_path.clone();
        let relay_task = tokio::spawn(async move {
            let (stream, _) = relay_listener.accept().await?;
            proxy_connection(stream, &relay_routes_path).await
        });

        let mut client = TcpStream::connect(relay_address).await?;
        client
            .write_all(b"GET /health HTTP/1.1\r\nHost: demo.example.com\r\n\r\n")
            .await?;
        client.shutdown().await?;
        let mut response = Vec::new();
        client.read_to_end(&mut response).await?;

        upstream_task.await.context("等待测试上游失败")??;
        relay_task.await.context("等待测试 relay 失败")??;
        let response = String::from_utf8(response).context("relay 返回了非 UTF-8 测试响应")?;
        assert!(response.ends_with("\r\n\r\nOK"));
        Ok(())
    }
}
