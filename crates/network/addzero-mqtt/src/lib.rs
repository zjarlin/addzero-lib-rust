#![forbid(unsafe_code)]

use rumqttc::{
    Client, ClientError, Connection, Event, LastWill, MqttOptions, Packet, QoS, RecvTimeoutError,
    SubscribeFilter, Transport,
};
use std::fmt;
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
use thiserror::Error;

pub type MqttResult<T> = Result<T, MqttError>;

#[derive(Debug, Error)]
pub enum MqttError {
    #[error("invalid mqtt configuration: {0}")]
    InvalidConfig(String),
    #[error("invalid mqtt message: {0}")]
    InvalidMessage(String),
    #[error("invalid mqtt subscription: {0}")]
    InvalidSubscription(String),
    #[error("mqtt client error: {0}")]
    Client(#[from] ClientError),
    #[error("failed to read `{path}`: {source}")]
    FileRead {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("mqtt receive channel is disconnected")]
    ReceiveDisconnected,
    #[error("failed to access internal state: {0}")]
    Internal(String),
    #[error("mqtt background thread panicked")]
    BackgroundThreadPanicked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttQoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl From<MqttQoS> for QoS {
    fn from(value: MqttQoS) -> Self {
        match value {
            MqttQoS::AtMostOnce => QoS::AtMostOnce,
            MqttQoS::AtLeastOnce => QoS::AtLeastOnce,
            MqttQoS::ExactlyOnce => QoS::ExactlyOnce,
        }
    }
}

impl From<QoS> for MqttQoS {
    fn from(value: QoS) -> Self {
        match value {
            QoS::AtMostOnce => MqttQoS::AtMostOnce,
            QoS::AtLeastOnce => MqttQoS::AtLeastOnce,
            QoS::ExactlyOnce => MqttQoS::ExactlyOnce,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: MqttQoS,
    pub retain: bool,
}

impl MqttMessage {
    pub fn builder(topic: impl Into<String>) -> MqttMessageBuilder {
        Self {
            topic: topic.into(),
            payload: Vec::new(),
            qos: MqttQoS::AtMostOnce,
            retain: false,
        }
    }

    pub fn validate(&self) -> MqttResult<()> {
        if self.topic.trim().is_empty() {
            return Err(MqttError::InvalidMessage(
                "topic cannot be blank".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn payload(mut self, value: impl Into<Vec<u8>>) -> Self {
        self.payload = value.into();
        self
    }

    pub fn payload_str(mut self, value: impl Into<String>) -> Self {
        self.payload = value.into().into_bytes();
        self
    }

    pub fn qos(mut self, value: MqttQoS) -> Self {
        self.qos = value;
        self
    }

    pub fn retain(mut self, value: bool) -> Self {
        self.retain = value;
        self
    }

    pub fn build(self) -> MqttResult<MqttMessage> {
        self.validate()?;
        Ok(self)
    }
}

pub type MqttMessageBuilder = MqttMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttReceivedMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: MqttQoS,
    pub retain: bool,
    pub duplicate: bool,
    pub packet_id: Option<u16>,
}

impl MqttReceivedMessage {
    pub fn payload_as_utf8_lossy(&self) -> String {
        String::from_utf8_lossy(&self.payload).into_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttSubscription {
    pub topic_filter: String,
    pub qos: MqttQoS,
}

impl MqttSubscription {
    pub fn new(topic_filter: impl Into<String>, qos: MqttQoS) -> Self {
        Self {
            topic_filter: topic_filter.into(),
            qos,
        }
    }

    pub fn validate(&self) -> MqttResult<()> {
        if self.topic_filter.trim().is_empty() {
            return Err(MqttError::InvalidSubscription(
                "topic_filter cannot be blank".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keep_alive_secs: u64,
    pub clean_session: bool,
    pub request_channel_capacity: usize,
    pub inflight: u16,
    pub connect_timeout_secs: u64,
    pub poll_timeout_ms: u64,
    pub use_tls: bool,
    pub ca_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub last_will: Option<MqttMessage>,
}

impl fmt::Debug for MqttConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MqttConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("client_id", &self.client_id)
            .field("username", &self.username)
            .field("password", &self.password.as_deref().map(|_| "***"))
            .field("keep_alive_secs", &self.keep_alive_secs)
            .field("clean_session", &self.clean_session)
            .field("request_channel_capacity", &self.request_channel_capacity)
            .field("inflight", &self.inflight)
            .field("connect_timeout_secs", &self.connect_timeout_secs)
            .field("poll_timeout_ms", &self.poll_timeout_ms)
            .field("use_tls", &self.use_tls)
            .field("ca_path", &self.ca_path)
            .field("client_cert_path", &self.client_cert_path)
            .field("client_key_path", &self.client_key_path)
            .field("last_will", &self.last_will)
            .finish()
    }
}

impl MqttConfig {
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

    pub fn port(mut self, value: u16) -> Self {
        self.port = value;
        self
    }

    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.username = Some(value.into());
        self
    }

    pub fn password(mut self, value: impl Into<String>) -> Self {
        self.password = Some(value.into());
        self
    }

    pub fn keep_alive_secs(mut self, value: u64) -> Self {
        self.keep_alive_secs = value;
        self
    }

    pub fn clean_session(mut self, value: bool) -> Self {
        self.clean_session = value;
        self
    }

    pub fn request_channel_capacity(mut self, value: usize) -> Self {
        self.request_channel_capacity = value;
        self
    }

    pub fn inflight(mut self, value: u16) -> Self {
        self.inflight = value;
        self
    }

    pub fn connect_timeout_secs(mut self, value: u64) -> Self {
        self.connect_timeout_secs = value;
        self
    }

    pub fn poll_timeout_ms(mut self, value: u64) -> Self {
        self.poll_timeout_ms = value;
        self
    }

    pub fn use_tls(mut self, value: bool) -> Self {
        self.use_tls = value;
        self
    }

    pub fn ca_path(mut self, value: impl Into<String>) -> Self {
        self.ca_path = Some(value.into());
        self.use_tls = true;
        self
    }

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

    pub fn last_will(mut self, value: MqttMessage) -> Self {
        self.last_will = Some(value);
        self
    }

    pub fn build(self) -> MqttResult<MqttConfig> {
        self.validate()?;
        Ok(self)
    }
}

pub type MqttConfigBuilder = MqttConfig;

pub struct MqttClient {
    config: MqttConfig,
    client: Client,
    receiver: Mutex<mpsc::Receiver<MqttReceivedMessage>>,
    stop: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl MqttClient {
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

    pub fn config(&self) -> &MqttConfig {
        &self.config
    }

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

    pub fn subscribe(&self, topic_filter: impl Into<String>, qos: MqttQoS) -> MqttResult<()> {
        let subscription = MqttSubscription::new(topic_filter, qos);
        subscription.validate()?;
        self.client
            .subscribe(subscription.topic_filter, subscription.qos.into())?;
        Ok(())
    }

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

    pub fn receive(&self) -> MqttResult<MqttReceivedMessage> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|_| MqttError::Internal("receive mutex is poisoned".to_owned()))?;
        receiver.recv().map_err(|_| MqttError::ReceiveDisconnected)
    }

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

impl From<rumqttc::Publish> for MqttReceivedMessage {
    fn from(value: rumqttc::Publish) -> Self {
        Self {
            topic: value.topic,
            payload: value.payload.to_vec(),
            qos: value.qos.into(),
            retain: value.retain,
            duplicate: value.dup,
            packet_id: (value.pkid != 0).then_some(value.pkid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mqtt_config_debug_masks_password_when_present() {
        let config = MqttConfig::builder("broker.example.com", "client-id")
            .username("alice")
            .password("broker-password");

        let debug = format!("{config:?}");

        assert!(debug.contains("password: Some(\"***\")"));
        assert!(!debug.contains("broker-password"));
    }
}
