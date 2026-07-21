# az-ssh

SSH 远程连接客户端，提供命令执行和 SFTP 文件传输功能。

## 功能

- **SSH 连接管理**：支持密码和私钥两种认证方式，可配置连接超时和读写超时
- **远程命令执行**：`execute_sync` 同步执行命令并获取 stdout/stderr/exit_code；`execute_stream` 支持逐行流式回调
- **SFTP 文件传输**：`upload_file` / `download_file` 支持单文件和目录的递归上传/下载，自动创建远程父目录
- **安全配置**：`SshConfig` 的 `Debug` 实现自动脱敏密码、私钥路径和密码短语
- **便捷函数**：提供 `connect`、`with_session`、`execute_sync`、`upload_file` 等顶层函数，无需手动管理会话生命周期
- **配置校验**：`SshConfig::validate()` 在连接前检查 host、username、port 和认证方式的合法性

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-ssh = { path = "../ssh" }             # workspace 内部引用
# 或发布后：
# az-ssh = "0.1"                             # crates.io 引用
```

## 用法

### 使用密码连接并执行命令

```rust,no_run
use az_ssh::client::{SshConfig, execute_sync};

fn main() -> anyhow::Result<()> {
let config = SshConfig::builder("192.168.1.100", "root")
    .password("your-password")
    .port(22)
    .build()?;

let result = execute_sync(&config, "uname -a")?;
if result.is_success() {
    println!("{}", result.stdout);
}
Ok(())
}
```

### 使用私钥连接并上传文件

```rust,no_run
use az_ssh::client::{SshConfig, SshSession};

fn main() -> anyhow::Result<()> {
let config = SshConfig::builder("example.com", "deploy")
    .private_key_path("~/.ssh/id_rsa")
    .build()?;

let session = SshSession::connect(config)?;
session.upload_file("./dist/app.tar.gz", "/opt/deploy/")?;
session.execute_sync("cd /opt/deploy && tar xzf app.tar.gz")?;
Ok(())
}
```

### 流式读取远程命令输出

```rust,no_run
use az_ssh::client::{SshConfig, execute_stream};

fn main() -> anyhow::Result<()> {
let config = SshConfig::builder("example.com", "admin")
    .password("secret")
    .build()?;

let result = execute_stream(&config, "tail -f /var/log/app.log", |line| {
    println!("[remote] {line}");
})?;
drop(result);
Ok(())
}
```

## 依赖的 crates

- `ssh2` - libssh2 的 Rust 绑定，提供 SSH 协议底层支持
- `anyhow` - 错误上下文与统一 `Result`
