use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use anyhow::{Context, Result};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        http::{HeaderValue, header},
    },
};

use crate::contracts::SyncWireMessage;

type SyncWsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct SyncWsConnection {
    pub writer: SyncWsWriter,
    pub reader: SyncWsReader,
}

impl SyncWsConnection {
    pub async fn connect(endpoint: impl AsRef<str>, token: Option<&str>) -> Result<Self> {
        let mut request = endpoint.as_ref().into_client_request()?;
        if let Some(token) = token.filter(|value| !value.trim().is_empty()) {
            let value = HeaderValue::from_str(&format!("Bearer {token}"))
                .context("invalid sync WebSocket auth header")?;
            request.headers_mut().insert(header::AUTHORIZATION, value);
        }
        let (stream, _) = connect_async(request).await?;
        let (writer, reader) = stream.split();
        Ok(Self {
            writer: SyncWsWriter { writer },
            reader: SyncWsReader { reader },
        })
    }
}

pub struct SyncWsWriter {
    writer: SplitSink<SyncWsStream, Message>,
}

impl SyncWsWriter {
    pub async fn send(&mut self, message: &SyncWireMessage) -> Result<()> {
        let text = serde_json::to_string(message).context("sync wire JSON failed")?;
        self.writer.send(Message::Text(text.into())).await?;
        Ok(())
    }
}

pub struct SyncWsReader {
    reader: SplitStream<SyncWsStream>,
}

impl SyncWsReader {
    pub async fn recv(&mut self) -> Result<Option<SyncWireMessage>> {
        while let Some(message) = self.reader.next().await {
            let message = message?;
            match message {
                Message::Text(text) => {
                    return serde_json::from_str(&text)
                        .map(Some)
                        .context("sync wire JSON failed");
                }
                Message::Binary(_) => {
                    return Ok(Some(SyncWireMessage::Error {
                        message: "binary WebSocket frames are not part of the sync protocol yet"
                            .to_string(),
                    }));
                }
                Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
                Message::Close(_) => return Ok(None),
            }
        }
        Ok(None)
    }
}
