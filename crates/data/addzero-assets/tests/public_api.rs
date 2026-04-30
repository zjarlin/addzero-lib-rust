use addzero_assets::{AssetKind, AssetService, AssetUpsert, SecretCipher};

#[tokio::test]
async fn public_api_should_store_assets_and_hash_content() {
    let service = AssetService::memory_only(None);
    let saved = service
        .upsert_asset(AssetUpsert {
            id: None,
            kind: AssetKind::Note,
            title: "自动标题".into(),
            body: "用户只发送采集内容".into(),
            tags: vec!["采集".into(), "笔记".into()],
            status: "active".into(),
            metadata: serde_json::json!({"source": "test"}),
        })
        .await
        .unwrap();
    assert_eq!(saved.kind, AssetKind::Note);
    assert!(!saved.content_hash.is_empty());
}

#[test]
fn public_api_should_encrypt_model_keys() {
    let cipher = SecretCipher::from_master_key("dev-secret").unwrap();
    let encrypted = cipher.encrypt("anthropic-key").unwrap();
    assert_eq!(cipher.decrypt(&encrypted).unwrap(), "anthropic-key");
}
