use az_ai_chat::{ChatError, ChatOptions, ChatResponse, Message, Role};

#[test]
fn message_constructors() {
    let sys = Message::system("be helpful");
    assert_eq!(sys.role, Role::System);
    assert_eq!(sys.content, "be helpful");

    let usr = Message::user("hello");
    assert_eq!(usr.role, Role::User);

    let asst = Message::assistant("hi there");
    assert_eq!(asst.role, Role::Assistant);
}

#[test]
fn chat_options_builder() {
    let opts = ChatOptions::new()
        .with_temperature(0.7)
        .with_max_tokens(256);
    assert_eq!(opts.temperature, Some(0.7));
    assert_eq!(opts.max_tokens, Some(256));
    assert!(opts.stop.is_none());
}

#[test]
fn message_serialization_roundtrip() {
    let msg = Message::user("test message");
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}

#[test]
fn role_serialization() {
    let json = serde_json::to_string(&Role::System).unwrap();
    assert_eq!(json, "\"system\"");
    let json = serde_json::to_string(&Role::User).unwrap();
    assert_eq!(json, "\"user\"");
    let json = serde_json::to_string(&Role::Assistant).unwrap();
    assert_eq!(json, "\"assistant\"");
}

#[test]
fn chat_response_deserialization() {
    let json = r#"{
        "content": "Hello!",
        "model": "gpt-4",
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
        "finish_reason": "stop"
    }"#;
    let resp: ChatResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.content, "Hello!");
    assert_eq!(resp.model, "gpt-4");
    assert_eq!(resp.usage.unwrap().total_tokens, 15);
    assert_eq!(resp.finish_reason.as_deref(), Some("stop"));
}

#[test]
fn chat_error_display() {
    let err = ChatError::InvalidConfig("empty api key".into());
    assert_eq!(err.to_string(), "invalid config: empty api key");

    let err = ChatError::ProviderError {
        code: 429,
        message: "rate limited".into(),
    };
    assert!(err.to_string().contains("429"));
}
