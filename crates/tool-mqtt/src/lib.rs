#![forbid(unsafe_code)]

use rumqttc::{
    Client, ClientError, Connection, Event, LastWill, MqttOptions, Packet, QoS, RecvTimeoutError,
    SubscribeFilter, Transport,
};
use std::fs;
use std::io;
use std::path::Path;
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
        MqttMessageBuilder {
            topic: Some(topic.into()),
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
}

#[derive(Debug, Clone)]
pub struct MqttMessageBuilder {
    topic: Option<String>,
    payload: Vec<u8>,
    qos: MqttQoS,
    retain: bool,
}

impl MqttMessageBuilder {
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
        let message = MqttMessage {
            topic: self.topic.unwrap_or_default(),
            payload: self.payload,
            qos: self.qos,
            retain: self.retain,
        };
        message.validate()?;
        Ok(message)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl MqttConfig {
    pub fn builder(host: impl Into<String>, client_id: impl Into<String>) -> MqttConfigBuilder {
        MqttConfigBuilder {
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
        if let Some(last_will) = &self.last_will {
            last_will.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MqttConfigBuilder {
    host: String,
    port: u16,
    client_id: String,
    username: Option<String>,
    password: Option<String>,
    keep_alive_secs: u64,
    clean_session: bool,
    request_channel_capacity: usize,
    inflight: u16,
    connect_timeout_secs: u64,
    poll_timeout_ms: u64,
    use_tls: bool,
    ca_path: Option<String>,
    client_cert_path: Option<String>,
    client_key_path: Option<String>,
    last_will: Option<MqttMessage>,
}

impl MqttConfigBuilder {
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
        let config = MqttConfig {
            host: self.host,
            port: self.port,
            client_id: self.client_id,
            username: self.username,
            password: self.password,
            keep_alive_secs: self.keep_alive_secs,
            clean_session: self.clean_session,
            request_channel_capacity: self.request_channel_capacity,
            inflight: self.inflight,
            connect_timeout_secs: self.connect_timeout_secs,
            poll_timeout_ms: self.poll_timeout_ms,
            use_tls: self.use_tls,
            ca_path: self.ca_path,
            client_cert_path: self.client_cert_path,
            client_key_path: self.client_key_path,
            last_will: self.last_will,
        };
        config.validate()?;
        Ok(config)
    }
}

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
            handle.join().map_err(|_| MqttError::BackgroundThreadPanicked)?;
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
        options.set_credentials(username.clone(), config.password.clone().unwrap_or_default());
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
    let ca = config
        .ca_path
        .as_ref()
        .map(|path| read_file_bytes(path))
        .transpose()?;
    let client_auth = match (&config.client_cert_path, &config.client_key_path) {
        (Some(cert_path), Some(key_path)) => Some((read_file_bytes(cert_path)?, read_file_bytes(key_path)?)),
        _ => None,
    };

    Ok(match (ca, client_auth) {
        (None, None) => Transport::tls_with_default_config(),
        (Some(ca), client_auth) => Transport::tls(ca, client_auth, None),
        (None, Some(_)) => {
            return Err(MqttError::InvalidConfig(
                "ca_path is required when client certificate authentication is configured"
                    .to_owned(),
            ))
        }
    })
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn mqtt_message_builder_validates_topic() {
        let error = MqttMessage::builder("   ").build().unwrap_err();
        assert!(matches!(error, MqttError::InvalidMessage(_)));
    }

    #[test]
    fn mqtt_config_builder_validates_auth_pairing() {
        let error = MqttConfig::builder("localhost", "client-a")
            .password("secret")
            .build()
            .unwrap_err();
        assert!(matches!(error, MqttError::InvalidConfig(_)));
    }

    #[test]
    fn build_options_sets_core_fields() {
        let last_will = MqttMessage::builder("system/last-will")
            .payload_str("bye")
            .qos(MqttQoS::AtLeastOnce)
            .retain(true)
            .build()
            .unwrap();
        let config = MqttConfig::builder("broker.example.com", "client-a")
            .port(2883)
            .username("user")
            .password("pass")
            .keep_alive_secs(30)
            .clean_session(false)
            .request_channel_capacity(32)
            .inflight(7)
            .last_will(last_will)
            .build()
            .unwrap();

        let options = build_options(&config).unwrap();
        assert_eq!(options.broker_address(), ("broker.example.com".to_owned(), 2883));
        assert_eq!(options.client_id(), "client-a".to_owned());
        assert_eq!(options.keep_alive(), Duration::from_secs(30));
        assert!(!options.clean_session());
        assert_eq!(options.request_channel_capacity(), 32);
        assert_eq!(options.inflight(), 7);

        let credentials = options.credentials().unwrap();
        assert_eq!(credentials.username, "user");
        assert_eq!(credentials.password, "pass");

        let last_will = options.last_will().unwrap();
        assert_eq!(last_will.topic, "system/last-will");
        assert_eq!(last_will.message.as_ref(), b"bye");
        assert_eq!(last_will.qos, QoS::AtLeastOnce);
        assert!(last_will.retain);
    }

    #[test]
    fn build_transport_uses_default_tls_when_no_files_are_provided() {
        let config = MqttConfig::builder("broker.example.com", "client-a")
            .use_tls(true)
            .build()
            .unwrap();

        let transport = build_transport(&config).unwrap();
        assert!(matches!(transport, Transport::Tls(_)));
    }

    #[test]
    fn build_transport_loads_custom_ca_and_client_auth() {
        let temp_dir = std::env::temp_dir().join(unique_suffix("tool-mqtt"));
        fs::create_dir_all(&temp_dir).unwrap();

        let ca_path = temp_dir.join("ca.pem");
        let cert_path = temp_dir.join("client.pem");
        let key_path = temp_dir.join("client.key");
        fs::write(&ca_path, b"ca-bytes").unwrap();
        fs::write(&cert_path, b"cert-bytes").unwrap();
        fs::write(&key_path, b"key-bytes").unwrap();

        let config = MqttConfig::builder("broker.example.com", "client-a")
            .ca_path(ca_path.display().to_string())
            .client_auth_paths(cert_path.display().to_string(), key_path.display().to_string())
            .build()
            .unwrap();

        let transport = build_transport(&config).unwrap();
        match transport {
            Transport::Tls(rumqttc::TlsConfiguration::Simple {
                ca, client_auth, ..
            }) => {
                assert_eq!(ca, b"ca-bytes".to_vec());
                let (cert, key) = client_auth.unwrap();
                assert_eq!(cert, b"cert-bytes".to_vec());
                assert_eq!(key, b"key-bytes".to_vec());
            }
            _ => panic!("expected rustls tls transport"),
        }

        let _ = fs::remove_file(ca_path);
        let _ = fs::remove_file(cert_path);
        let _ = fs::remove_file(key_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn collect_messages_returns_empty_when_count_is_zero() {
        let (sender, receiver) = mpsc::channel();
        drop(sender);

        let receiver = Mutex::new(receiver);
        let result = {
            let client = Client::from_sender(flume::unbounded().0);
            let mqtt_client = MqttClient {
                config: MqttConfig::builder("localhost", "client-a").build().unwrap(),
                client,
                receiver,
                stop: Arc::new(AtomicBool::new(true)),
                worker: Mutex::new(None),
            };
            mqtt_client.collect_messages(0, Duration::from_millis(1))
        };

        assert_eq!(result.unwrap(), Vec::<MqttReceivedMessage>::new());
    }

    fn unique_suffix(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{prefix}-{nanos}")
    }
}
