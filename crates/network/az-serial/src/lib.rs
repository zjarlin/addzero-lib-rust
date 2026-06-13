//! 跨平台串口通信工具。
//!
//! 提供可移植的 [`SerialPort`] 抽象，用于通过串口（UART/RS-232）连接读写数据，
//! 支持可配置的波特率、校验位、停止位和流控制。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_serial::config::{BaudRate, Parity, SerialConfig, StopBits};
//! use az_serial::port::SerialPort;
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

automod::dir!(pub "src");
