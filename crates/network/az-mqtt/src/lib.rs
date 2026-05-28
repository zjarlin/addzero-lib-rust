//! MQTT 客户端封装，基于 rumqttc 提供同步的发布/订阅接口。
//!
//! # 核心类型
//!
//! - [`MqttConfig`] — MQTT 连接配置，支持 TLS（CA 证书 + 客户端证书双向认证），
//!   凭证字段在 `Debug` 输出中自动脱敏。
//! - [`MqttClient`] — 客户端主入口，内部维护后台轮询线程，通过 [`MqttMessage`] 发布消息，
//!   通过 `subscribe()` / `subscribe_many()` 订阅主题，通过 `receive()` / `receive_timeout()` 获取收到的消息。
//! - [`MqttMessage`] — 待发布消息构建器，支持 topic、payload（字节或字符串）、QoS 和 retain 标志。
//! - [`MqttReceivedMessage`] — 收到的消息，包含 topic、payload、QoS、retain、duplicate 及 packet_id。
//! - [`MqttQoS`] — QoS 等级枚举（AtMostOnce / AtLeastOnce / ExactlyOnce），与 rumqttc 双向转换。
//! - [`MqttError`] — 统一错误类型。
//!
//! # 关键功能
//!
//! - **Builder 模式**：`MqttConfig::builder(host, client_id)` 和 `MqttMessage::builder(topic)` 提供链式配置。
//! - **TLS 支持**：通过 `ca_path()` 和 `client_auth_paths()` 配置 CA 和客户端证书，自动启用 TLS。
//! - **Last Will**：通过 `last_will()` 设置遗嘱消息。
//! - **批量接收**：`collect_messages(max, timeout)` 在超时内收集最多 N 条消息。
//! - **自动断开清理**：`MqttClient` 实现 `Drop`，析构时自动停止后台线程并断开连接。
//! - **禁止 unsafe**：整个 crate 使用 `#![forbid(unsafe_code)]`。
//!
//! # 快速开始
//!
//! ```rust,no_run
//! use az_mqtt::{MqttConfig, MqttClient, MqttMessage, MqttQoS};
//! use std::time::Duration;
//!
//! let config = MqttConfig::builder("broker.example.com", "my-client")
//!     .port(1883)
//!     .build()
//!     .unwrap();
//!
//! let client = MqttClient::connect(config).unwrap();
//!
//! client.subscribe("sensors/temperature", MqttQoS::AtLeastOnce).unwrap();
//!
//! client.publish_str("sensors/humidity", "65.3", MqttQoS::AtMostOnce, false).unwrap();
//!
//! if let Ok(Some(msg)) = client.receive_timeout(Duration::from_secs(5)) {
//!     println!("收到: {} -> {}", msg.topic, msg.payload_as_utf8_lossy());
//! }
//!
//! client.disconnect().unwrap();
//! ```
#![forbid(unsafe_code)]

use az_derive_aliases::{
    apply, error, impl_from_match, plain_code_enum, plain_eq, plain_eq_redacted,
};
use rumqttc::{
    Client, ClientError, Connection, Event, LastWill, MqttOptions, Packet, QoS, RecvTimeoutError,
    SubscribeFilter, Transport,
};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Once;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// MQTT crate 的统一结果类型。
pub type MqttResult<T> = Result<T, MqttError>;

/// MQTT 配置、消息、订阅和后台轮询过程中可能返回的错误。
///
/// 该类型保留 `rumqttc` 和证书文件 IO 的源错误链；后台线程内部轮询错误会被吞掉并继续轮询，显式 API 错误会通过该类型返回。
#[apply(error)]
pub enum MqttError {
    /// 连接配置不完整或字段组合非法。
    #[error("invalid mqtt configuration: {0}")]
    InvalidConfig(String),
    /// 待发布消息不满足本地校验。
    #[error("invalid mqtt message: {0}")]
    InvalidMessage(String),
    /// 订阅主题过滤器为空或订阅列表为空。
    #[error("invalid mqtt subscription: {0}")]
    InvalidSubscription(String),
    /// `rumqttc` 客户端 API 返回的错误。
    #[error("mqtt client error: {0}")]
    Client(#[from] ClientError),
    /// 读取 TLS CA、客户端证书或私钥文件失败。
    #[error("failed to read `{path}`: {source}")]
    FileRead {
        path: String,
        #[source]
        source: io::Error,
    },
    /// 接收通道已断开，通常表示后台轮询线程已经结束。
    #[error("mqtt receive channel is disconnected")]
    ReceiveDisconnected,
    /// 内部互斥锁等状态访问失败。
    #[error("failed to access internal state: {0}")]
    Internal(String),
    /// 显式断开时后台线程 panic。
    #[error("mqtt background thread panicked")]
    BackgroundThreadPanicked,
}

/// MQTT 服务质量等级。
///
/// `code()` / `from_code()` 使用 snake_case 机器码，并与 `rumqttc::QoS` 保持双向转换。
#[apply(plain_code_enum)]
pub enum MqttQoS {
    /// QoS 0，最多一次投递。
    AtMostOnce,
    /// QoS 1，至少一次投递。
    AtLeastOnce,
    /// QoS 2，恰好一次投递。
    ExactlyOnce,
}

impl_from_match!(MqttQoS => QoS {
    MqttQoS::AtMostOnce => Self::AtMostOnce,
    MqttQoS::AtLeastOnce => Self::AtLeastOnce,
    MqttQoS::ExactlyOnce => Self::ExactlyOnce,
});

impl_from_match!(QoS => MqttQoS {
    QoS::AtMostOnce => Self::AtMostOnce,
    QoS::AtLeastOnce => Self::AtLeastOnce,
    QoS::ExactlyOnce => Self::ExactlyOnce,
});

/// 待发布的 MQTT 消息。
///
/// 该类型同时用于 Last Will 配置；topic 不允许为空，payload 可以是任意字节。
#[apply(plain_eq)]
pub struct MqttMessage {
    /// 发布主题。
    pub topic: String,
    /// 消息负载字节。
    pub payload: Vec<u8>,
    /// 发布 QoS。
    pub qos: MqttQoS,
    /// 是否让 broker 保留该消息。
    pub retain: bool,
}

impl MqttMessage {
    /// 创建消息构建器，默认 QoS 为 `AtMostOnce`，`retain` 为 `false`。
    pub fn builder(topic: impl Into<String>) -> MqttMessageBuilder {
        Self {
            topic: topic.into(),
            payload: Vec::new(),
            qos: MqttQoS::AtMostOnce,
            retain: false,
        }
    }

    /// 校验消息是否满足本地发送前置条件。
    ///
    /// 当前只校验 topic 非空，不校验 topic 是否符合 broker 侧 ACL 或通配符规则。
    pub fn validate(&self) -> MqttResult<()> {
        if self.topic.trim().is_empty() {
            return Err(MqttError::InvalidMessage(
                "topic cannot be blank".to_owned(),
            ));
        }
        Ok(())
    }

    /// 设置二进制负载。
    pub fn payload(mut self, value: impl Into<Vec<u8>>) -> Self {
        self.payload = value.into();
        self
    }

    /// 设置 UTF-8 字符串负载。
    pub fn payload_str(mut self, value: impl Into<String>) -> Self {
        self.payload = value.into().into_bytes();
        self
    }

    /// 设置发布 QoS。
    pub fn qos(mut self, value: MqttQoS) -> Self {
        self.qos = value;
        self
    }

    /// 设置 retain 标志。
    pub fn retain(mut self, value: bool) -> Self {
        self.retain = value;
        self
    }

    /// 完成构建并执行本地消息校验。
    pub fn build(self) -> MqttResult<MqttMessage> {
        self.validate()?;
        Ok(self)
    }
}

/// `MqttMessage` 采用自身作为轻量 builder。
pub type MqttMessageBuilder = MqttMessage;

/// 从 broker 收到的 MQTT 发布消息。
#[apply(plain_eq)]
pub struct MqttReceivedMessage {
    /// 消息主题。
    pub topic: String,
    /// 原始负载字节。
    pub payload: Vec<u8>,
    /// broker 投递该消息时的 QoS。
    pub qos: MqttQoS,
    /// broker 标记的 retain 状态。
    pub retain: bool,
    /// 是否为重复投递。
    pub duplicate: bool,
    /// MQTT packet id；QoS 0 通常没有有效 packet id。
    pub packet_id: Option<u16>,
}

impl MqttReceivedMessage {
    /// 以 UTF-8 lossy 方式读取负载。
    ///
    /// 非 UTF-8 字节会被替换字符处理，适合日志和调试；协议级解析应直接使用 `payload`。
    pub fn payload_as_utf8_lossy(&self) -> String {
        String::from_utf8_lossy(&self.payload).into_owned()
    }
}

/// MQTT 订阅请求。
///
/// `topic_filter` 可以包含 MQTT 通配符，具体合法性和权限仍由 broker 判断。
#[apply(plain_eq)]
pub struct MqttSubscription {
    /// 订阅主题过滤器。
    pub topic_filter: String,
    /// 订阅 QoS。
    pub qos: MqttQoS,
}

impl MqttSubscription {
    /// 创建订阅请求。
    pub fn new(topic_filter: impl Into<String>, qos: MqttQoS) -> Self {
        Self {
            topic_filter: topic_filter.into(),
            qos,
        }
    }

    /// 校验订阅过滤器的最小本地约束。
    pub fn validate(&self) -> MqttResult<()> {
        if self.topic_filter.trim().is_empty() {
            return Err(MqttError::InvalidSubscription(
                "topic_filter cannot be blank".to_owned(),
            ));
        }
        Ok(())
    }
}

/// MQTT 客户端连接配置。
///
/// 密码和客户端私钥路径在 `Debug` 输出中会被脱敏；配置 TLS 文件路径时会自动启用 TLS。
#[apply(plain_eq_redacted)]
pub struct MqttConfig {
    /// broker 主机名或 IP。
    pub host: String,
    /// broker 端口，默认 1883。
    pub port: u16,
    /// MQTT client id。
    pub client_id: String,
    /// 可选用户名。
    pub username: Option<String>,
    /// 可选密码，调试输出会脱敏。
    #[debug(skip)]
    pub password: Option<String>,
    /// keep-alive 秒数。
    pub keep_alive_secs: u64,
    /// 是否使用 clean session。
    pub clean_session: bool,
    /// `rumqttc` 请求通道容量。
    pub request_channel_capacity: usize,
    /// 最大 inflight 包数量。
    pub inflight: u16,
    /// 连接超时秒数。
    pub connect_timeout_secs: u64,
    /// 后台轮询线程的单次接收超时毫秒数。
    pub poll_timeout_ms: u64,
    /// 是否启用 TLS。
    pub use_tls: bool,
    /// CA 证书路径；存在时会自动启用 TLS。
    pub ca_path: Option<String>,
    /// 客户端证书路径，用于双向认证。
    pub client_cert_path: Option<String>,
    /// 客户端私钥路径，用于双向认证，调试输出会脱敏。
    #[debug(skip)]
    pub client_key_path: Option<String>,
    /// Last Will 消息。
    pub last_will: Option<MqttMessage>,
}

impl MqttConfig {
    /// 创建 MQTT 配置构建器。
    ///
    /// 默认端口 1883、clean session、keep-alive 60 秒，不启用 TLS。
    pub fn builder(host: impl Into<String>, client_id: impl Into<String>) -> MqttConfigBuilder {
        Self {
            host: host.into(),
            port: 1883,
            client_id: client_id.into(),
            username: None,
            password: None,
            keep_alive_secs: 60,
            clean_session: true,
            request_channel_capacity: 10,
            inflight: 100,
            connect_timeout_secs: 5,
            poll_timeout_ms: 250,
            use_tls: false,
            ca_path: None,
            client_cert_path: None,
            client_key_path: None,
            last_will: None,
        }
    }

    /// 校验连接配置的本地约束。
    ///
    /// 该方法不连接 broker，只检查字段为空、成对认证字段、TLS 开关和 Last Will 消息合法性。
    pub fn validate(&self) -> MqttResult<()> {
        if self.host.trim().is_empty() {
            return Err(MqttError::InvalidConfig("host cannot be blank".to_owned()));
        }
        if self.client_id.trim().is_empty() {
            return Err(MqttError::InvalidConfig(
                "client_id cannot be blank".to_owned(),
            ));
        }
        if self.port == 0 {
            return Err(MqttError::InvalidConfig(
                "port must be greater than zero".to_owned(),
            ));
        }
        if self.request_channel_capacity == 0 {
            return Err(MqttError::InvalidConfig(
                "request_channel_capacity must be greater than zero".to_owned(),
            ));
        }
        if self.inflight == 0 {
            return Err(MqttError::InvalidConfig(
                "inflight must be greater than zero".to_owned(),
            ));
        }
        if self.connect_timeout_secs == 0 {
            return Err(MqttError::InvalidConfig(
                "connect_timeout_secs must be greater than zero".to_owned(),
            ));
        }
        if self.poll_timeout_ms == 0 {
            return Err(MqttError::InvalidConfig(
                "poll_timeout_ms must be greater than zero".to_owned(),
            ));
        }
        if self.password.is_some() && self.username.is_none() {
            return Err(MqttError::InvalidConfig(
                "username is required when password is set".to_owned(),
            ));
        }
        if self.client_cert_path.is_some() ^ self.client_key_path.is_some() {
            return Err(MqttError::InvalidConfig(
                "client_cert_path and client_key_path must be set together".to_owned(),
            ));
        }
        if !self.use_tls
            && (self.ca_path.is_some()
                || self.client_cert_path.is_some()
                || self.client_key_path.is_some())
        {
            return Err(MqttError::InvalidConfig(
                "use_tls must be enabled when tls file paths are configured".to_owned(),
            ));
        }
        if let Some(last_will) = &self.last_will {
            last_will.validate()?;
        }
        Ok(())
    }

    /// 设置 broker 端口。
    pub fn port(mut self, value: u16) -> Self {
        self.port = value;
        self
    }

    /// 设置认证用户名。
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.username = Some(value.into());
        self
    }

    /// 设置认证密码。
    pub fn password(mut self, value: impl Into<String>) -> Self {
        self.password = Some(value.into());
        self
    }

    /// 设置 keep-alive 秒数。
    pub fn keep_alive_secs(mut self, value: u64) -> Self {
        self.keep_alive_secs = value;
        self
    }

    /// 设置 clean session。
    pub fn clean_session(mut self, value: bool) -> Self {
        self.clean_session = value;
        self
    }

    /// 设置请求通道容量。
    pub fn request_channel_capacity(mut self, value: usize) -> Self {
        self.request_channel_capacity = value;
        self
    }

    /// 设置最大 inflight 包数量。
    pub fn inflight(mut self, value: u16) -> Self {
        self.inflight = value;
        self
    }

    /// 设置连接超时秒数。
    pub fn connect_timeout_secs(mut self, value: u64) -> Self {
        self.connect_timeout_secs = value;
        self
    }

    /// 设置后台轮询线程的接收超时毫秒数。
    pub fn poll_timeout_ms(mut self, value: u64) -> Self {
        self.poll_timeout_ms = value;
        self
    }

    /// 显式设置是否启用 TLS。
    pub fn use_tls(mut self, value: bool) -> Self {
        self.use_tls = value;
        self
    }

    /// 设置 CA 证书路径并自动启用 TLS。
    pub fn ca_path(mut self, value: impl Into<String>) -> Self {
        self.ca_path = Some(value.into());
        self.use_tls = true;
        self
    }

    /// 设置客户端证书和私钥路径并自动启用 TLS。
    pub fn client_auth_paths(
        mut self,
        cert_path: impl Into<String>,
        key_path: impl Into<String>,
    ) -> Self {
        self.client_cert_path = Some(cert_path.into());
        self.client_key_path = Some(key_path.into());
        self.use_tls = true;
        self
    }

    /// 设置 Last Will 消息。
    pub fn last_will(mut self, value: MqttMessage) -> Self {
        self.last_will = Some(value);
        self
    }

    /// 完成构建并执行本地配置校验。
    pub fn build(self) -> MqttResult<MqttConfig> {
        self.validate()?;
        Ok(self)
    }
}

/// `MqttConfig` 采用自身作为轻量 builder。
pub type MqttConfigBuilder = MqttConfig;

/// 同步 MQTT 客户端。
///
/// 客户端内部持有 `rumqttc::Client` 和一个后台轮询线程；收到的发布消息通过内部通道交给 `receive` 系列方法。
pub struct MqttClient {
    config: MqttConfig,
    client: Client,
    receiver: Mutex<mpsc::Receiver<MqttReceivedMessage>>,
    stop: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl MqttClient {
    /// 连接 broker 并启动后台轮询线程。
    ///
    /// 返回成功表示本地 client 已创建并开始轮询；broker 侧认证、网络断开等运行期问题仍可能在后续 API 中暴露。
    pub fn connect(config: MqttConfig) -> MqttResult<Self> {
        config.validate()?;

        let options = build_options(&config)?;
        let (client, mut connection) = Client::new(options, config.request_channel_capacity);

        let mut network_options = connection.eventloop.network_options();
        network_options.set_connection_timeout(config.connect_timeout_secs);
        connection.eventloop.set_network_options(network_options);

        let (sender, receiver) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let worker = spawn_connection_worker(
            connection,
            config.poll_timeout_ms,
            Arc::clone(&stop),
            sender,
        );

        Ok(Self {
            config,
            client,
            receiver: Mutex::new(receiver),
            stop,
            worker: Mutex::new(Some(worker)),
        })
    }

    /// 返回客户端持有的连接配置。
    pub fn config(&self) -> &MqttConfig {
        &self.config
    }

    /// 发布一条 MQTT 消息。
    pub fn publish(&self, message: &MqttMessage) -> MqttResult<()> {
        message.validate()?;
        self.client.publish(
            message.topic.clone(),
            message.qos.into(),
            message.retain,
            message.payload.clone(),
        )?;
        Ok(())
    }

    /// 构建并发布一条 UTF-8 字符串消息。
    pub fn publish_str(
        &self,
        topic: impl Into<String>,
        payload: impl Into<String>,
        qos: MqttQoS,
        retain: bool,
    ) -> MqttResult<()> {
        let message = MqttMessage::builder(topic)
            .payload_str(payload)
            .qos(qos)
            .retain(retain)
            .build()?;
        self.publish(&message)
    }

    /// 订阅单个 topic filter。
    pub fn subscribe(&self, topic_filter: impl Into<String>, qos: MqttQoS) -> MqttResult<()> {
        let subscription = MqttSubscription::new(topic_filter, qos);
        subscription.validate()?;
        self.client
            .subscribe(subscription.topic_filter, subscription.qos.into())?;
        Ok(())
    }

    /// 批量订阅多个 topic filter。
    ///
    /// 空订阅列表会返回 `MqttError::InvalidSubscription`。
    pub fn subscribe_many(
        &self,
        subscriptions: impl IntoIterator<Item = MqttSubscription>,
    ) -> MqttResult<()> {
        let mut filters = Vec::new();
        for subscription in subscriptions {
            subscription.validate()?;
            filters.push(SubscribeFilter::new(
                subscription.topic_filter,
                subscription.qos.into(),
            ));
        }
        if filters.is_empty() {
            return Err(MqttError::InvalidSubscription(
                "at least one subscription is required".to_owned(),
            ));
        }
        self.client.subscribe_many(filters)?;
        Ok(())
    }

    /// 取消订阅单个 topic filter。
    pub fn unsubscribe(&self, topic_filter: impl Into<String>) -> MqttResult<()> {
        let topic_filter = topic_filter.into();
        if topic_filter.trim().is_empty() {
            return Err(MqttError::InvalidSubscription(
                "topic_filter cannot be blank".to_owned(),
            ));
        }
        self.client.unsubscribe(topic_filter)?;
        Ok(())
    }

    /// 阻塞接收下一条发布消息。
    ///
    /// 如果后台轮询线程已经结束并关闭通道，则返回 `MqttError::ReceiveDisconnected`。
    pub fn receive(&self) -> MqttResult<MqttReceivedMessage> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|_| MqttError::Internal("receive mutex is poisoned".to_owned()))?;
        receiver.recv().map_err(|_| MqttError::ReceiveDisconnected)
    }

    /// 在指定超时时间内接收下一条发布消息。
    ///
    /// 超时返回 `Ok(None)`；通道断开返回 `MqttError::ReceiveDisconnected`。
    pub fn receive_timeout(&self, timeout: Duration) -> MqttResult<Option<MqttReceivedMessage>> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|_| MqttError::Internal("receive mutex is poisoned".to_owned()))?;
        match receiver.recv_timeout(timeout) {
            Ok(message) => Ok(Some(message)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(MqttError::ReceiveDisconnected),
        }
    }

    /// 在超时窗口内收集最多 `max_messages` 条消息。
    ///
    /// `max_messages` 为 0 时立即返回空列表；该方法不会延长总超时时间。
    pub fn collect_messages(
        &self,
        max_messages: usize,
        timeout: Duration,
    ) -> MqttResult<Vec<MqttReceivedMessage>> {
        if max_messages == 0 {
            return Ok(Vec::new());
        }

        let deadline = Instant::now() + timeout;
        let mut messages = Vec::with_capacity(max_messages);
        while messages.len() < max_messages {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break;
            };
            match self.receive_timeout(remaining)? {
                Some(message) => messages.push(message),
                None => break,
            }
        }
        Ok(messages)
    }

    /// 主动断开连接并等待后台轮询线程退出。
    ///
    /// `Drop` 也会做清理，但显式调用可以把后台线程 panic 映射为可见错误。
    pub fn disconnect(&self) -> MqttResult<()> {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.client.disconnect();

        let mut worker = self
            .worker
            .lock()
            .map_err(|_| MqttError::Internal("worker mutex is poisoned".to_owned()))?;
        if let Some(handle) = worker.take() {
            handle
                .join()
                .map_err(|_| MqttError::BackgroundThreadPanicked)?;
        }
        Ok(())
    }
}

impl Drop for MqttClient {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.client.disconnect();
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(handle) = worker.take() {
                let _ = handle.join();
            }
        }
    }
}

fn spawn_connection_worker(
    mut connection: Connection,
    poll_timeout_ms: u64,
    stop: Arc<AtomicBool>,
    sender: mpsc::Sender<MqttReceivedMessage>,
) -> JoinHandle<()> {
    let poll_timeout = Duration::from_millis(poll_timeout_ms);
    thread::spawn(move || {
        while !stop.load(Ordering::SeqCst) {
            match connection.recv_timeout(poll_timeout) {
                Ok(Ok(Event::Incoming(Packet::Publish(publish)))) => {
                    if sender.send(MqttReceivedMessage::from(publish)).is_err() {
                        break;
                    }
                }
                Ok(Ok(_)) => {}
                Ok(Err(_)) => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    })
}

fn build_options(config: &MqttConfig) -> MqttResult<MqttOptions> {
    let mut options = MqttOptions::new(config.client_id.clone(), config.host.clone(), config.port);
    options.set_keep_alive(Duration::from_secs(config.keep_alive_secs));
    options.set_clean_session(config.clean_session);
    options.set_request_channel_capacity(config.request_channel_capacity);
    options.set_inflight(config.inflight);

    if let Some(username) = &config.username {
        options.set_credentials(
            username.clone(),
            config.password.clone().unwrap_or_default(),
        );
    }

    if let Some(last_will) = &config.last_will {
        options.set_last_will(LastWill::new(
            last_will.topic.clone(),
            last_will.payload.clone(),
            last_will.qos.into(),
            last_will.retain,
        ));
    }

    if config.use_tls {
        options.set_transport(build_transport(config)?);
    }

    Ok(options)
}

fn build_transport(config: &MqttConfig) -> MqttResult<Transport> {
    let ca = config.ca_path.as_ref().map(read_file_bytes).transpose()?;
    let client_auth = match (&config.client_cert_path, &config.client_key_path) {
        (Some(cert_path), Some(key_path)) => {
            Some((read_file_bytes(cert_path)?, read_file_bytes(key_path)?))
        }
        _ => None,
    };

    Ok(match (ca, client_auth) {
        (None, None) => {
            install_rustls_provider();
            Transport::tls_with_default_config()
        }
        (Some(ca), client_auth) => Transport::tls(ca, client_auth, None),
        (None, Some(_)) => {
            return Err(MqttError::InvalidConfig(
                "ca_path is required when client certificate authentication is configured"
                    .to_owned(),
            ));
        }
    })
}

fn install_rustls_provider() {
    static INSTALL_RUSTLS_PROVIDER: Once = Once::new();

    INSTALL_RUSTLS_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn read_file_bytes(path: impl AsRef<Path>) -> MqttResult<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).map_err(|source| MqttError::FileRead {
        path: path.display().to_string(),
        source,
    })
}

impl_from_match!(rumqttc::Publish => MqttReceivedMessage {
    value => MqttReceivedMessage {
        topic: value.topic,
        payload: value.payload.to_vec(),
        qos: value.qos.into(),
        retain: value.retain,
        duplicate: value.dup,
        packet_id: (value.pkid != 0).then_some(value.pkid),
    }
});

#[cfg(test)]
mod debug_redaction_tests {
    use super::MqttConfig;

    #[test]
    fn mqtt_config_debug_does_not_leak_credentials() {
        let config = MqttConfig::builder("localhost", "client-1")
            .username("alice")
            .password("mqtt-secret")
            .ca_path("/tmp/ca.pem")
            .client_auth_paths("/tmp/client.crt", "/tmp/client.key")
            .build()
            .expect("mqtt config should build");

        let output = format!("{config:?}");
        assert!(output.contains("localhost"));
        assert!(!output.contains("mqtt-secret"));
        assert!(!output.contains("/tmp/client.key"));
        assert!(output.contains("/tmp/ca.pem"));
        assert!(output.contains("/tmp/client.crt"));
    }
}
