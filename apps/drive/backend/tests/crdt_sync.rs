//! Integration tests for the CRDT WebSocket sync endpoint.
//!
//! Starts a local axum server backed by in-memory stores, connects
//! simulated clients, and verifies full-sync and incremental-reconnect
//! scenarios.

use std::sync::Arc;

use axum::{Router, extract::WebSocketUpgrade, routing::get};
use az_crdt::document::LineCrdtDocument;
use az_drive_app::ws::{CrdtSyncState, handle_crdt_sync};
use az_drive_store::api::{InMemoryDriveMetadataStore, InMemoryDriveObjectStore};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use base64::Engine as _;

async fn test_server(state: Arc<CrdtSyncState>) -> u16 {
    let app = Router::new().route(
        "/ws/sync",
        get(|ws: WebSocketUpgrade| async move {
            ws.on_upgrade(move |socket| handle_crdt_sync(socket, state))
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    port
}

async fn connect(
    port: u16,
) -> (
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    u64,
) {
    let url = format!("ws://127.0.0.1:{port}/ws/sync");
    let (mut ws, _) = connect_async(&url).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.to_text().unwrap();
    let ack: Value = serde_json::from_str(text).unwrap();
    let peer_id = ack["peer_id"].as_u64().unwrap();
    (ws, peer_id)
}

fn open_msg(path: &str) -> Message {
    Message::Text(format!(r#"{{"type":"open","remote_path":"{path}"}}"#).into())
}

fn open_with_version_msg(path: &str, version: &str) -> Message {
    Message::Text(
        format!(r#"{{"type":"open","remote_path":"{path}","base_version":"{version}"}}"#).into(),
    )
}

fn update_msg(path: &str, update_b64: &str) -> Message {
    Message::Text(
        format!(r#"{{"type":"update","remote_path":"{path}","update":"{update_b64}","base_version":null}}"#).into(),
    )
}

fn close_msg(path: &str) -> Message {
    Message::Text(format!(r#"{{"type":"close","remote_path":"{path}"}}"#).into())
}

/// Extracts the `snapshot` base64 field from an `opened` message.
fn opened_snapshot(opened: &Value) -> &str {
    opened["snapshot"].as_str().unwrap()
}

/// Extracts the `version` field from an `opened` or `update` message.
fn msg_version(msg: &Value) -> &str {
    msg["version"].as_str().unwrap()
}

/// Extracts the `update` base64 field from an `update` or `opened` message.
fn msg_update(msg: &Value) -> &str {
    msg["update"].as_str().unwrap()
}

// ── tests ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn two_clients_sync_document() {
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let state = Arc::new(CrdtSyncState::new(
        metadata.clone(),
        objects.clone(),
        "test".to_owned(),
    ));
    let port = test_server(state.clone()).await;

    let (mut client_a, _pa) = connect(port).await;
    let (mut client_b, _pb) = connect(port).await;

    // A opens "notes/readme.md" → gets full snapshot.
    client_a.send(open_msg("notes/readme.md")).await.unwrap();
    let opened_a: Value = read_json(&mut client_a).await;
    assert_eq!(opened_a["type"], "opened");
    let snapshot_b64 = opened_snapshot(&opened_a);

    // B opens the same file.
    client_b.send(open_msg("notes/readme.md")).await.unwrap();
    let _opened_b: Value = read_json(&mut client_b).await;

    // A inserts text, exports update, sends it.
    let doc_a = restore_doc(snapshot_b64);
    doc_a.insert_text(0, "Hello from client A\n").unwrap();
    let update_a_b64 = export_update(&doc_a);
    client_a
        .send(update_msg("notes/readme.md", &update_a_b64))
        .await
        .unwrap();

    // B receives the broadcast update.
    let b_update: Value = read_json(&mut client_b).await;
    assert_eq!(b_update["type"], "update");
    assert_eq!(b_update["remote_path"], "notes/readme.md");

    // B imports update and verifies.
    let doc_b = restore_doc(snapshot_b64);
    doc_b
        .import_update(b64decode(msg_update(&b_update)))
        .unwrap();
    assert_eq!(doc_b.text(), "Hello from client A\n");

    // Cleanup.
    client_a.send(close_msg("notes/readme.md")).await.unwrap();
    client_b.send(close_msg("notes/readme.md")).await.unwrap();
    let _ = client_a.close(None).await;
    let _ = client_b.close(None).await;
}

#[tokio::test]
async fn incremental_reconnect() {
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let state = Arc::new(CrdtSyncState::new(
        metadata.clone(),
        objects.clone(),
        "test".to_owned(),
    ));
    let port = test_server(state.clone()).await;

    // Client A connects, opens file, pushes an edit.
    let (mut client_a, _pa) = connect(port).await;
    client_a.send(open_msg("doc.txt")).await.unwrap();
    let opened: Value = read_json(&mut client_a).await;
    let snap_b64 = opened_snapshot(&opened);
    let v1 = msg_version(&opened).to_owned();

    let doc_a = restore_doc(snap_b64);
    doc_a.insert_text(0, "line one\n").unwrap();
    let u1_b64 = export_update(&doc_a);
    client_a.send(update_msg("doc.txt", &u1_b64)).await.unwrap();

    // Read back the broadcast (echo to self via broadcast_to_others — actually
    // this won't echo to self; let's just check the update was accepted by
    // looking at the server state via another open).
    client_a.send(close_msg("doc.txt")).await.unwrap();

    // Client B connects fresh, opens with base_version=v1 → should get
    // incremental update instead of full snapshot.
    let (mut client_b, _pb) = connect(port).await;
    client_b
        .send(open_with_version_msg("doc.txt", &v1))
        .await
        .unwrap();
    let reopened: Value = read_json(&mut client_b).await;
    assert_eq!(reopened["type"], "opened");
    // Should have update, not snapshot (since server has v1 cached).
    assert!(
        reopened["snapshot"].is_null(),
        "expected incremental update, not full snapshot"
    );
    assert!(reopened["update"].is_string(), "expected update field");

    // Apply the update to a fresh doc at v1.
    let doc_b = restore_doc(snap_b64);
    doc_b
        .import_update(b64decode(msg_update(&reopened)))
        .unwrap();
    assert_eq!(doc_b.text(), "line one\n");

    client_b.send(close_msg("doc.txt")).await.unwrap();
    let _ = client_b.close(None).await;
    let _ = client_a.close(None).await;
}

// ── helpers ───────────────────────────────────────────────────────────

async fn read_json(
    ws: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin),
) -> Value {
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.to_text().unwrap();
    serde_json::from_str(text).unwrap()
}

fn restore_doc(snapshot_b64: &str) -> LineCrdtDocument {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(snapshot_b64)
        .unwrap();
    LineCrdtDocument::from_snapshot(bytes).unwrap()
}

fn export_update(doc: &LineCrdtDocument) -> String {
    let update = doc.export_all_updates().unwrap();
    base64::engine::general_purpose::STANDARD.encode(update.as_bytes())
}

fn b64decode(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD.decode(s).unwrap()
}

fn put_binary_msg(path: &str, offset: u64, data_b64: &str, is_last: bool) -> Message {
    Message::Text(
        format!(
            r#"{{"type":"put_binary","remote_path":"{path}","offset":{offset},"data":"{data_b64}","is_last":{is_last}}}"#
        )
        .into(),
    )
}

#[tokio::test]
async fn binary_upload_single_chunk() {
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let state = Arc::new(CrdtSyncState::new(
        metadata.clone(),
        objects.clone(),
        "test".to_owned(),
    ));
    let port = test_server(state.clone()).await;

    let (mut client, _peer) = connect(port).await;

    let payload = b"hello binary world";
    let data_b64 = base64::engine::general_purpose::STANDARD.encode(payload);

    client
        .send(put_binary_msg("images/photo.png", 0, &data_b64, true))
        .await
        .unwrap();

    let ack: Value = read_json(&mut client).await;
    assert_eq!(ack["type"], "binary_ack");
    assert_eq!(ack["remote_path"], "images/photo.png");
    assert_eq!(ack["size_bytes"].as_u64().unwrap(), payload.len() as u64);
    // Verify the hash matches.
    let expected_hash = az_drive_core::api::content_hash(payload);
    assert_eq!(ack["hash"].as_str().unwrap(), expected_hash);

    let _ = client.close(None).await;
}

// ── GetBinary / List tests ────────────────────────────────────────────

#[tokio::test]
async fn binary_roundtrip() {
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let state = Arc::new(CrdtSyncState::new(
        metadata.clone(),
        objects.clone(),
        "test".to_owned(),
    ));
    let port = test_server(state.clone()).await;

    let (mut client, _peer) = connect(port).await;

    // Upload a binary file.
    let payload = b"binary roundtrip test data";
    let data_b64 = base64::engine::general_purpose::STANDARD.encode(payload);
    client
        .send(put_binary_msg("data.bin", 0, &data_b64, true))
        .await
        .unwrap();
    let ack: Value = read_json(&mut client).await;
    assert_eq!(ack["type"], "binary_ack");

    // Download it back.
    client
        .send(Message::Text(
            r#"{"type":"get_binary","remote_path":"data.bin"}"#.into(),
        ))
        .await
        .unwrap();

    let mut downloaded = Vec::new();
    loop {
        let chunk: Value = read_json(&mut client).await;
        assert_eq!(chunk["type"], "binary_chunk");
        let chunk_data = b64decode(chunk["data"].as_str().unwrap());
        downloaded.extend_from_slice(&chunk_data);
        if chunk["is_last"].as_bool().unwrap() {
            break;
        }
    }
    assert_eq!(downloaded, payload);

    let _ = client.close(None).await;
}

#[tokio::test]
async fn list_entries() {
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let state = Arc::new(CrdtSyncState::new(
        metadata.clone(),
        objects.clone(),
        "test".to_owned(),
    ));
    let port = test_server(state.clone()).await;

    let (mut client, _peer) = connect(port).await;

    // Upload two files first so we have something to list.
    let d1 = base64::engine::general_purpose::STANDARD.encode(b"aaa");
    client
        .send(put_binary_msg("notes/a.txt", 0, &d1, true))
        .await
        .unwrap();
    let _: Value = read_json(&mut client).await;

    let d2 = base64::engine::general_purpose::STANDARD.encode(b"bbb");
    client
        .send(put_binary_msg("notes/b.txt", 0, &d2, true))
        .await
        .unwrap();
    let _: Value = read_json(&mut client).await;

    // List all entries.
    client
        .send(Message::Text(r#"{"type":"list"}"#.into()))
        .await
        .unwrap();
    let result: Value = read_json(&mut client).await;
    assert_eq!(result["type"], "list_result");
    let entries = result["entries"].as_array().unwrap();
    assert!(entries.len() >= 2);

    // List with prefix.
    client
        .send(Message::Text(r#"{"type":"list","prefix":"notes/"}"#.into()))
        .await
        .unwrap();
    let result2: Value = read_json(&mut client).await;
    let entries2 = result2["entries"].as_array().unwrap();
    assert_eq!(entries2.len(), 2);

    let _ = client.close(None).await;
}
