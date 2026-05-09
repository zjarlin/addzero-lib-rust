//! Serial port configuration types.

use serde::{Deserialize, Serialize};

/// Standard baud rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaudRate {
    /// 0 baud (invalid / disconnected).
    Baud0,
    /// 4800 baud.
    Baud4800,
    /// 9600 baud.
    Baud9600,
    /// 19200 baud.
    Baud19200,
    /// 38400 baud.
    Baud38400,
    /// 57600 baud.
    Baud57600,
    /// 115200 baud.
    Baud115200,
    /// 230400 baud.
    Baud230400,
    /// 460800 baud.
    Baud460800,
    /// 921600 baud.
    Baud921600,
    /// Custom baud rate.
    Custom(u32),
}

impl BaudRate {
    /// Get the numeric value of this baud rate.
    pub fn value(&self) -> u32 {
        match self {
            Self::Baud0 => 0,
            Self::Baud4800 => 4800,
            Self::Baud9600 => 9600,
            Self::Baud19200 => 19200,
            Self::Baud38400 => 38400,
            Self::Baud57600 => 57600,
            Self::Baud115200 => 115200,
            Self::Baud230400 => 230400,
            Self::Baud460800 => 460800,
            Self::Baud921600 => 921600,
            Self::Custom(v) => *v,
        }
    }
}

/// Parity checking mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parity {
    /// No parity bit.
    None,
    /// Even parity.
    Even,
    /// Odd parity.
    Odd,
    /// Mark parity (always 1).
    Mark,
    /// Space parity (always 0).
    Space,
}

/// Number of stop bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopBits {
    /// 1 stop bit.
    One,
    /// 2 stop bits.
    Two,
}

/// Hardware/software flow control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowControl {
    /// No flow control.
    None,
    /// Hardware RTS/CTS.
    Hardware,
    /// Software XON/XOFF.
    Software,
}

/// Serial port configuration parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerialConfig {
    /// Baud rate.
    pub baud_rate: BaudRate,
    /// Data bits per character (5, 6, 7, or 8).
    pub data_bits: u8,
    /// Parity checking mode.
    pub parity: Parity,
    /// Number of stop bits.
    pub stop_bits: StopBits,
    /// Flow control mode.
    pub flow_control: FlowControl,
    /// Read timeout in milliseconds (0 = non-blocking).
    pub read_timeout_ms: u64,
    /// Write timeout in milliseconds (0 = non-blocking).
    pub write_timeout_ms: u64,
}

impl SerialConfig {
    /// Create a new configuration with the given baud rate and sensible defaults:
    /// 8 data bits, no parity, 1 stop bit, no flow control.
    pub fn new(baud_rate: BaudRate) -> Self {
        Self {
            baud_rate,
            data_bits: 8,
            parity: Parity::None,
            stop_bits: StopBits::One,
            flow_control: FlowControl::None,
            read_timeout_ms: 0,
            write_timeout_ms: 0,
        }
    }

    /// Set the number of data bits (5, 6, 7, or 8).
    pub fn with_data_bits(mut self, bits: u8) -> Self {
        self.data_bits = bits;
        self
    }

    /// Set the parity mode.
    pub fn with_parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Set the stop bits.
    pub fn with_stop_bits(mut self, stop: StopBits) -> Self {
        self.stop_bits = stop;
        self
    }

    /// Set the flow control mode.
    pub fn with_flow_control(mut self, fc: FlowControl) -> Self {
        self.flow_control = fc;
        self
    }

    /// Set the read timeout in milliseconds.
    pub fn with_read_timeout(mut self, ms: u64) -> Self {
        self.read_timeout_ms = ms;
        self
    }

    /// Set the write timeout in milliseconds.
    pub fn with_write_timeout(mut self, ms: u64) -> Self {
        self.write_timeout_ms = ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baud_rate_values() {
        assert_eq!(BaudRate::Baud9600.value(), 9600);
        assert_eq!(BaudRate::Baud115200.value(), 115200);
        assert_eq!(BaudRate::Custom(256000).value(), 256000);
        assert_eq!(BaudRate::Baud0.value(), 0);
    }

    #[test]
    fn config_builder_defaults() {
        let config = SerialConfig::new(BaudRate::Baud115200);
        assert_eq!(config.baud_rate, BaudRate::Baud115200);
        assert_eq!(config.data_bits, 8);
        assert_eq!(config.parity, Parity::None);
        assert_eq!(config.stop_bits, StopBits::One);
        assert_eq!(config.flow_control, FlowControl::None);
    }

    #[test]
    fn config_builder_chaining() {
        let config = SerialConfig::new(BaudRate::Baud9600)
            .with_data_bits(7)
            .with_parity(Parity::Even)
            .with_stop_bits(StopBits::Two)
            .with_flow_control(FlowControl::Hardware)
            .with_read_timeout(1000);

        assert_eq!(config.data_bits, 7);
        assert_eq!(config.parity, Parity::Even);
        assert_eq!(config.stop_bits, StopBits::Two);
        assert_eq!(config.flow_control, FlowControl::Hardware);
        assert_eq!(config.read_timeout_ms, 1000);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = SerialConfig::new(BaudRate::Baud115200).with_parity(Parity::None);
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SerialConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn parity_serialization() {
        assert_eq!(serde_json::to_string(&Parity::None).unwrap(), "\"None\"");
        assert_eq!(serde_json::to_string(&Parity::Even).unwrap(), "\"Even\"");
    }
}
