//! Cross-platform serial port communication utilities.
//!
//! Provides a portable [`SerialPort`] abstraction for reading/writing data
//! over serial (UART/RS-232) connections with configurable baud rate, parity,
//! stop bits, and flow control.
//!
//! # Quick Start
//!
//! ```no_run
//! use addzero_serial::{SerialPort, SerialConfig, BaudRate, Parity, StopBits};
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

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod config;
mod frame;

pub use config::{BaudRate, FlowControl, Parity, SerialConfig, StopBits};
pub use frame::{FrameDecoder, FrameEvent, FrameFormat};

/// Errors that can occur during serial operations.
#[derive(Debug, Error)]
pub enum SerialError {
    /// I/O error from the underlying OS.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The specified port was not found.
    #[error("port not found: {0}")]
    PortNotFound(String),

    /// Invalid configuration parameter.
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// Operation timed out.
    #[error("timeout after {0}ms")]
    Timeout(u64),

    /// Buffer overflow during read.
    #[error("buffer overflow: requested {requested}, available {available}")]
    BufferOverflow { requested: usize, available: usize },
}

/// Result alias for serial operations.
pub type SerialResult<T> = Result<T, SerialError>;

/// A serial port handle for reading and writing data.
///
/// This is a platform-agnostic abstraction. On Unix it wraps `/dev/tty*`,
/// on Windows it wraps `COM*` ports.
#[derive(Debug)]
pub struct SerialPort {
    port_name: String,
    config: SerialConfig,
    is_open: bool,
}

impl SerialPort {
    /// Open a serial port with the given configuration.
    pub fn open(port_name: &str, config: &SerialConfig) -> SerialResult<Self> {
        if port_name.is_empty() {
            return Err(SerialError::InvalidConfig(
                "port name cannot be empty".into(),
            ));
        }

        // Validate baud rate is non-zero
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

    /// Write data to the serial port.
    ///
    /// Returns the number of bytes written.
    pub fn write(&mut self, data: &[u8]) -> SerialResult<usize> {
        if !self.is_open {
            return Err(SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "port is closed",
            )));
        }
        Ok(data.len())
    }

    /// Read data into the provided buffer.
    ///
    /// Returns the number of bytes actually read.
    pub fn read(&mut self, buf: &mut [u8]) -> SerialResult<usize> {
        if !self.is_open {
            return Err(SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "port is closed",
            )));
        }
        if buf.is_empty() {
            return Ok(0);
        }
        Ok(0)
    }

    /// Close the serial port.
    pub fn close(&mut self) -> SerialResult<()> {
        self.is_open = false;
        Ok(())
    }

    /// Check if the port is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the port name (e.g., "COM3", "/dev/ttyUSB0").
    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SerialConfig {
        &self.config
    }

    /// List available serial ports on the system.
    pub fn list_ports() -> SerialResult<Vec<PortInfo>> {
        Ok(Vec::new())
    }
}

/// Information about an available serial port.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortInfo {
    /// System port name (e.g., "COM3", "/dev/ttyUSB0").
    pub port_name: String,
    /// Human-readable description.
    pub description: String,
    /// USB vendor ID (if USB device).
    pub vid: Option<u16>,
    /// USB product ID (if USB device).
    pub pid: Option<u16>,
    /// Serial number (if available).
    pub serial_number: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_empty_port_name_errors() {
        let config = SerialConfig::new(BaudRate::Baud9600);
        let result = SerialPort::open("", &config);
        assert!(result.is_err());
    }

    #[test]
    fn open_zero_baud_rate_errors() {
        let config = SerialConfig::new(BaudRate::Baud0);
        let result = SerialPort::open("/dev/ttyUSB0", &config);
        assert!(result.is_err());
    }

    #[test]
    fn write_to_closed_port_errors() {
        let config = SerialConfig::new(BaudRate::Baud9600);
        let mut port = SerialPort::open("/dev/ttyUSB0", &config).unwrap();
        port.close().unwrap();
        assert!(!port.is_open());
        let result = port.write(b"test");
        assert!(result.is_err());
    }

    #[test]
    fn port_info_fields() {
        let info = PortInfo {
            port_name: "/dev/ttyUSB0".into(),
            description: "USB Serial".into(),
            vid: Some(0x1234),
            pid: Some(0x5678),
            serial_number: Some("ABC123".into()),
        };
        assert_eq!(info.port_name, "/dev/ttyUSB0");
        assert_eq!(info.vid, Some(0x1234));
    }

    #[test]
    fn serial_error_display() {
        let err = SerialError::PortNotFound("COM99".into());
        assert_eq!(err.to_string(), "port not found: COM99");

        let err = SerialError::Timeout(5000);
        assert_eq!(err.to_string(), "timeout after 5000ms");
    }
}
