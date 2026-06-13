//! 串口配置类型。

use az_derive_aliases::{apply, serde_code_enum, serde_eq, serde_eq_copy};

/// 常用串口波特率。
#[apply(serde_eq_copy)]
pub enum BaudRate {
    /// 0 波特；在本 crate 中表示非法或未连接配置。
    Baud0,
    /// 4800 波特。
    Baud4800,
    /// 9600 波特。
    Baud9600,
    /// 19200 波特。
    Baud19200,
    /// 38400 波特。
    Baud38400,
    /// 57600 波特。
    Baud57600,
    /// 115200 波特。
    Baud115200,
    /// 230400 波特。
    Baud230400,
    /// 460800 波特。
    Baud460800,
    /// 921600 波特。
    Baud921600,
    /// 自定义波特率。
    Custom(u32),
}

impl BaudRate {
    /// 返回波特率的数值形式。
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

/// 串口校验位模式。
#[apply(serde_code_enum)]
pub enum Parity {
    /// 不使用校验位。
    None,
    /// 偶校验。
    Even,
    /// 奇校验。
    Odd,
    /// 标记校验，校验位恒为 1。
    Mark,
    /// 空格校验，校验位恒为 0。
    Space,
}

/// 串口停止位数量。
#[apply(serde_code_enum)]
pub enum StopBits {
    /// 1 个停止位。
    One,
    /// 2 个停止位。
    Two,
}

/// 串口硬件或软件流控模式。
#[apply(serde_code_enum)]
pub enum FlowControl {
    /// 不使用流控。
    None,
    /// 硬件 RTS/CTS 流控。
    Hardware,
    /// 软件 XON/XOFF 流控。
    Software,
}

/// 串口连接配置参数。
#[apply(serde_eq)]
pub struct SerialConfig {
    /// 波特率。
    pub baud_rate: BaudRate,
    /// 每个字符的数据位数量，通常为 5、6、7 或 8。
    pub data_bits: u8,
    /// 校验位模式。
    pub parity: Parity,
    /// 停止位数量。
    pub stop_bits: StopBits,
    /// 流控模式。
    pub flow_control: FlowControl,
    /// 读取超时时间，单位毫秒；`0` 表示非阻塞。
    pub read_timeout_ms: u64,
    /// 写入超时时间，单位毫秒；`0` 表示非阻塞。
    pub write_timeout_ms: u64,
}

impl SerialConfig {
    /// 使用指定波特率创建配置，并采用常见默认值：
    ///
    /// 8 数据位、无校验、1 停止位、无流控。
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

    /// 设置数据位数量，通常为 5、6、7 或 8。
    pub fn with_data_bits(mut self, bits: u8) -> Self {
        self.data_bits = bits;
        self
    }

    /// 设置校验位模式。
    pub fn with_parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// 设置停止位数量。
    pub fn with_stop_bits(mut self, stop: StopBits) -> Self {
        self.stop_bits = stop;
        self
    }

    /// 设置流控模式。
    pub fn with_flow_control(mut self, fc: FlowControl) -> Self {
        self.flow_control = fc;
        self
    }

    /// 设置读取超时时间，单位毫秒。
    pub fn with_read_timeout(mut self, ms: u64) -> Self {
        self.read_timeout_ms = ms;
        self
    }

    /// 设置写入超时时间，单位毫秒。
    pub fn with_write_timeout(mut self, ms: u64) -> Self {
        self.write_timeout_ms = ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BaudRate, FlowControl, Parity, SerialConfig, StopBits};

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
        assert_eq!(Parity::None.code(), "none");
        assert_eq!(serde_json::to_string(&Parity::None).unwrap(), "\"none\"");
        assert_eq!(serde_json::to_string(&Parity::Even).unwrap(), "\"even\"");
    }

    #[test]
    fn stop_bits_and_flow_control_use_snake_case_codes() {
        assert_eq!(StopBits::One.code(), "one");
        assert_eq!(FlowControl::Hardware.code(), "hardware");
        assert_eq!(
            serde_json::to_string(&FlowControl::Software).unwrap(),
            "\"software\""
        );
    }
}
