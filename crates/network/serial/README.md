# az-serial

跨平台串口通信工具，提供可移植的串口读写和帧解码抽象。

## 功能

- **串口读写**：通过 `SerialPort` 打开 `/dev/tty*`（Unix）或 `COM*`（Windows）端口，进行同步读写
- **可配置参数**：支持自定义波特率（4800–921600 或自定义值）、数据位、校验位、停止位和流控制
- **帧解码**：`FrameDecoder` 支持三种工业常见帧格式——定长帧、分隔符帧和长度前缀帧
- **串口枚举**：`SerialPort::list_ports()` 列出系统可用串口及其 USB VID/PID 信息
- **配置序列化**：`SerialConfig` 可通过 serde 进行 JSON 序列化/反序列化，方便持久化配置
- **统一错误上下文**：公开操作返回 `anyhow::Result`，配置和 I/O 失败直接带上下文返回

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-serial = { path = "../serial" }       # workspace 内部引用
# 或发布后：
# az-serial = "0.1"                          # crates.io 引用
```

## 用法

### 打开串口并读写数据

```rust
use az_serial::config::{BaudRate, Parity, SerialConfig, StopBits};
use az_serial::port::SerialPort;

let config = SerialConfig::new(BaudRate::Baud115200)
    .with_parity(Parity::None)
    .with_stop_bits(StopBits::One)
    .with_data_bits(8);

let mut port = SerialPort::open("/dev/ttyUSB0", &config)?;
port.write(b"AT\r\n")?;

let mut buf = [0u8; 256];
let n = port.read(&mut buf)?;
println!("Received: {:?}", &buf[..n]);
port.close()?;
```

### 使用帧解码器解析串口数据流

```rust
use az_serial::frame::{FrameDecoder, FrameEvent, FrameFormat};

let mut decoder = FrameDecoder::new(FrameFormat::Delimiter(vec![0x0A]));
decoder.push(b"Hello");
assert!(decoder.poll().is_none());
decoder.push(b"\n");
assert_eq!(decoder.poll(), Some(FrameEvent::Frame(b"Hello".to_vec())));
```

## 依赖的 crates

- `serde` - 序列化/反序列化支持
- `serde_json` - JSON 处理（用于配置序列化）
- `anyhow` - 错误上下文与统一 `Result`
- `tokio` - 异步运行时（预留异步支持）
