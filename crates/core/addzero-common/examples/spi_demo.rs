#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

mod notify_spi {
    use super::{Arc, BTreeMap, Error, fmt};

    #[derive(Debug, Clone)]
    pub struct NotifyRequest {
        pub target: String,
        pub title: String,
        pub body: String,
    }

    impl NotifyRequest {
        pub fn new(
            target: impl Into<String>,
            title: impl Into<String>,
            body: impl Into<String>,
        ) -> Self {
            Self {
                target: target.into(),
                title: title.into(),
                body: body.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum NotifyError {
        EmptyBody,
        ProviderNotFound(String),
    }

    impl fmt::Display for NotifyError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::EmptyBody => write!(f, "notification body must not be empty"),
                Self::ProviderNotFound(provider) => {
                    write!(f, "provider `{provider}` is not registered")
                }
            }
        }
    }

    impl Error for NotifyError {}

    pub trait MessageSenderSpi: Send + Sync {
        fn send(&self, request: &NotifyRequest) -> Result<(), NotifyError>;
    }

    pub trait MessageSenderFactorySpi: Send + Sync {
        fn provider_key(&self) -> &'static str;
        fn create(&self) -> Arc<dyn MessageSenderSpi>;
    }

    #[derive(Default)]
    pub struct MessageSenderRegistry {
        factories: BTreeMap<String, Arc<dyn MessageSenderFactorySpi>>,
    }

    impl MessageSenderRegistry {
        pub fn register<F>(&mut self, factory: F) -> &mut Self
        where
            F: MessageSenderFactorySpi + 'static,
        {
            let key = factory.provider_key().to_owned();
            self.factories.insert(key, Arc::new(factory));
            self
        }

        pub fn available_providers(&self) -> Vec<&str> {
            self.factories.keys().map(String::as_str).collect()
        }

        pub fn resolve(&self, provider: &str) -> Result<Arc<dyn MessageSenderSpi>, NotifyError> {
            let factory = self
                .factories
                .get(provider)
                .ok_or_else(|| NotifyError::ProviderNotFound(provider.to_owned()))?;
            Ok(factory.create())
        }
    }

    pub struct NotificationService {
        sender: Arc<dyn MessageSenderSpi>,
    }

    impl NotificationService {
        pub fn new(sender: Arc<dyn MessageSenderSpi>) -> Self {
            Self { sender }
        }

        pub fn send_welcome(
            &self,
            target: impl Into<String>,
            body: impl Into<String>,
        ) -> Result<(), NotifyError> {
            let body = body.into();
            if body.trim().is_empty() {
                return Err(NotifyError::EmptyBody);
            }

            let request = NotifyRequest::new(target, "Welcome", body);
            self.sender.send(&request)
        }
    }
}

mod console_plugin {
    use super::Arc;
    use super::notify_spi::{
        MessageSenderFactorySpi, MessageSenderSpi, NotifyError, NotifyRequest,
    };

    pub struct ConsoleMessageSender;

    impl MessageSenderSpi for ConsoleMessageSender {
        fn send(&self, request: &NotifyRequest) -> Result<(), NotifyError> {
            println!(
                "[console] target={} title={} body={}",
                request.target, request.title, request.body
            );
            Ok(())
        }
    }

    pub struct ConsoleMessageSenderFactory;

    impl MessageSenderFactorySpi for ConsoleMessageSenderFactory {
        fn provider_key(&self) -> &'static str {
            "console"
        }

        fn create(&self) -> Arc<dyn MessageSenderSpi> {
            Arc::new(ConsoleMessageSender)
        }
    }
}

mod feishu_plugin {
    use super::Arc;
    use super::notify_spi::{
        MessageSenderFactorySpi, MessageSenderSpi, NotifyError, NotifyRequest,
    };

    pub struct FeishuBotSender {
        webhook_url: String,
    }

    impl MessageSenderSpi for FeishuBotSender {
        fn send(&self, request: &NotifyRequest) -> Result<(), NotifyError> {
            println!(
                "[feishu] webhook={} target={} title={} body={}",
                self.webhook_url, request.target, request.title, request.body
            );
            Ok(())
        }
    }

    pub struct FeishuBotSenderFactory {
        webhook_url: String,
    }

    impl FeishuBotSenderFactory {
        pub fn new(webhook_url: impl Into<String>) -> Self {
            Self {
                webhook_url: webhook_url.into(),
            }
        }
    }

    impl MessageSenderFactorySpi for FeishuBotSenderFactory {
        fn provider_key(&self) -> &'static str {
            "feishu"
        }

        fn create(&self) -> Arc<dyn MessageSenderSpi> {
            Arc::new(FeishuBotSender {
                webhook_url: self.webhook_url.clone(),
            })
        }
    }
}

use console_plugin::ConsoleMessageSenderFactory;
use feishu_plugin::FeishuBotSenderFactory;
use notify_spi::{MessageSenderRegistry, NotificationService};

fn build_service(provider: &str) -> Result<NotificationService, Box<dyn Error>> {
    let mut registry = MessageSenderRegistry::default();
    registry
        .register(ConsoleMessageSenderFactory)
        .register(FeishuBotSenderFactory::new(
            "https://open.feishu.cn/open-apis/bot/v2/hook/demo",
        ));

    println!("available providers: {:?}", registry.available_providers());

    // If you later adopt shaku, this composition root is where it belongs:
    // shaku can wire the chosen implementation, but it does not auto-discover
    // plugin crates the way Java ServiceLoader does.
    let sender = registry.resolve(provider)?;
    Ok(NotificationService::new(sender))
}

fn main() -> Result<(), Box<dyn Error>> {
    let provider = std::env::var("MESSAGE_PROVIDER").unwrap_or_else(|_| "console".to_owned());
    let service = build_service(&provider)?;

    service.send_welcome("alice", "Your workspace is ready.")?;
    service.send_welcome(
        "bob",
        "You can swap providers without changing service code.",
    )?;
    Ok(())
}
