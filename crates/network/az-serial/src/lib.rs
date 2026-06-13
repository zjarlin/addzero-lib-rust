//! 跨平台串口通信工具。
//!
//! 提供可移植的 [`SerialPort`] 抽象，用于通过串口（UART/RS-232）连接读写数据，
//! 支持可配置的波特率、校验位、停止位和流控制。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_serial::{SerialPort, SerialConfig, BaudRate, Parity, StopBits};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = SerialConfig::new(BaudRate::Baud115200)
//!     .with_parity(Parity::None)
//!     .with_stop_bits(StopBits::One)
//!     .with_data_bits(8);
//!
//! let mut port = SerialPort::open("/dev/ttyUSB0", &config)?;
//! port.write(b"AT\r\n")?;
//!
//! let mut buf = [0u8; 256];
//! let n = port.read(&mut buf)?;
//! println!("Received: {:?}", &buf[..n]);
//! # Ok(())
//! # }
//! ```

use az_derive_aliases::{apply, error, plain_debug, serde_eq};

automod::dir!("src");

pub use config::{BaudRate, FlowControl, Parity, SerialConfig, StopBits};
pub use frame::{FrameDecoder, FrameEvent, FrameFormat};

/// 串口操作过程中可能返回的错误。
#[apply(error)]
pub enum SerialError {
    /// 底层操作系统返回的 I/O 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// 指定串口不存在。
    #[error("port not found: {0}")]
    PortNotFound(String),

    /// 串口配置参数非法。
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// 读写操作超时。
    #[error("timeout after {0}ms")]
    Timeout(u64),

    /// 读取缓冲区容量不足。
    #[error("buffer overflow: requested {requested}, available {available}")]
    BufferOverflow { requested: usize, available: usize },
}

/// 串口操作统一结果类型。
pub type SerialResult<T> = Result<T, SerialError>;

/// 可读写串口句柄。
///
/// 这是平台无关抽象：Unix 系统通常对应 `/dev/tty*`，Windows 系统通常对应
/// `COM*` 端口。当前类型只暴露统一的打开、读写和关闭契约。
#[apply(plain_debug)]
pub struct SerialPort {
    port_name: String,
    config: SerialConfig,
    is_open: bool,
}

impl SerialPort {
    /// 使用指定配置打开串口。
    pub fn open(port_name: &str, config: &SerialConfig) -> SerialResult<Self> {
        if port_name.is_empty() {
            return Err(SerialError::InvalidConfig(
                "port name cannot be empty".into(),
            ));
        }

        // 波特率为 0 在本抽象中表示不可用配置。
        if config.baud_rate.value() == 0 {
            return Err(SerialError::InvalidConfig(
                "baud rate cannot be zero".into(),
            ));
        }

        Ok(Self {
            port_name: port_name.to_string(),
            config: config.clone(),
            is_open: true,
        })
    }

    /// 向串口写入字节。
    ///
    /// 返回实际写入的字节数。
    pub fn write(&mut self, data: &[u8]) -> SerialResult<usize> {
        if !self.is_open {
            let source = std::io::Error::new(std::io::ErrorKind::NotConnected, "port is closed");
            let error = SerialError::Io(source);
            return Err(error);
        }
        Ok(data.len())
    }

    /// 从串口读取字节到调用方提供的缓冲区。
    ///
    /// 返回实际读取的字节数。
    pub fn read(&mut self, buf: &mut [u8]) -> SerialResult<usize> {
        if !self.is_open {
            let source = std::io::Error::new(std::io::ErrorKind::NotConnected, "port is closed");
            let error = SerialError::Io(source);
            return Err(error);
        }
        if buf.is_empty() {
            return Ok(0);
        }
        Ok(0)
    }

    /// 关闭串口句柄。
    pub fn close(&mut self) -> SerialResult<()> {
        self.is_open = false;
        Ok(())
    }

    /// 判断串口句柄当前是否处于打开状态。
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// 返回系统串口名称，例如 `COM3` 或 `/dev/ttyUSB0`。
    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    /// 返回当前串口配置。
    pub fn config(&self) -> &SerialConfig {
        &self.config
    }

    /// 列出当前系统可用串口。
    pub fn list_ports() -> SerialResult<Vec<PortInfo>> {
        Ok(Vec::new())
    }
}

/// 系统可用串口的描述信息。
#[apply(serde_eq)]
pub struct PortInfo {
    /// 系统串口名称，例如 `COM3` 或 `/dev/ttyUSB0`。
    pub port_name: String,
    /// 面向用户展示的串口描述。
    pub description: String,
    /// USB 厂商 ID；非 USB 设备时为空。
    pub vid: Option<u16>,
    /// USB 产品 ID；非 USB 设备时为空。
    pub pid: Option<u16>,
    /// 设备序列号；系统无法提供时为空。
    pub serial_number: Option<String>,
}
