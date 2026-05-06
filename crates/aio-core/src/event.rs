//! 全局事件总线 — 插件间解耦通信的核心机制。
//!
//! 支持发布/订阅模式（Pub/Sub），事件携带类型化 payload。
//! 线程安全，适用于内核各层及插件间异步通信。

use std::any::Any;
use std::sync::Arc;
use tokio::sync::broadcast;

/// A type-erased event payload.
pub type EventPayload = Arc<dyn Any + Send + Sync>;

/// An event with a topic tag and optional payload.
#[derive(Clone)]
pub struct Event {
    pub topic: String,
    pub payload: Option<EventPayload>,
}

/// Global event bus using tokio broadcast channels.
pub struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new event bus with the given buffer capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event to all subscribers of the topic.
    /// Returns the number of subscribers that received the event.
    pub fn publish(&self, topic: impl Into<String>, payload: Option<EventPayload>) -> usize {
        let event = Event {
            topic: topic.into(),
            payload,
        };
        self.tx.send(event).unwrap_or(0)
    }

    /// Subscribe to all events. Use `Event::topic` to filter.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        // Default buffer: 256 events before lagging receivers are dropped
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pub_sub_round_trip() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish("test.topic", None);

        let event = rx.recv().await.expect("should receive event");
        assert_eq!(event.topic, "test.topic");
        assert!(event.payload.is_none());
    }
}
