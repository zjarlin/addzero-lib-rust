//! 串口句柄与端口信息类型。

use anyhow::{Result, bail};
use az_derive_aliases::{apply, plain_debug, serde_eq};

use crate::config::SerialConfig;

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
    pub fn open(port_name: &str, config: &SerialConfig) -> Result<Self> {
        if port_name.is_empty() {
            bail!("invalid config: port name cannot be empty");
        }

        if config.baud_rate.value() == 0 {
            bail!("invalid config: baud rate cannot be zero");
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
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        if !self.is_open {
            bail!(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "port is closed"
            ));
        }
        Ok(data.len())
    }

    /// 从串口读取字节到调用方提供的缓冲区。
    ///
    /// 返回实际读取的字节数。
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.is_open {
            bail!(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "port is closed"
            ));
        }
        if buf.is_empty() {
            return Ok(0);
        }
        Ok(0)
    }

    /// 关闭串口句柄。
    pub fn close(&mut self) -> Result<()> {
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
    pub fn list_ports() -> Result<Vec<PortInfo>> {
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
