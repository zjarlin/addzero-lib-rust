# az-rustfs

S3 兼容对象存储客户端抽象。封装 `rust-s3`，提供同步/异步上传、下载、分片上传、进度追踪等功能，同时内置纯内存实现用于测试。

## 功能

- `S3StorageClient` trait：统一的存储客户端接口（bucket 管理、对象 CRUD、预签名 URL）
- `Rustfs` facade：统一入口，支持默认配置、显式配置和注入式工厂构造
- `BlockingS3StorageClient`：基于 `rust-s3` 的同步 S3 实现
- `InMemoryS3StorageClient`：纯内存的测试用实现
- `S3StorageClientFactory`：可注入的客户端工厂模式
- 智能上传：自动判断是否使用分片上传（`should_use_multipart_upload`），自动计算最优分片大小
- 分片上传进度追踪：`UploadProgressListener` trait + `SpeedTrackingProgressListener` 实现
- 断点续传支持：`resume_or_upload` 自动跳过已完成分片
- 预签名 URL 生成：`get_presigned_object_url`
- 工具函数：`guess_content_type`、`build_list_request`、`ensure_bucket`、`put_object_bytes`、`put_object_file` 等

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-rustfs = { path = "../rustfs" }       # workspace 内部引用
# 或发布后：
# az-rustfs = "0.1"                         # crates.io 引用
```

## 用法

```rust,no_run
use az_rustfs::object_ops::{ensure_bucket, get_object, put_object_bytes};
use az_rustfs::rustfs::Rustfs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Rustfs::default_client();
    let bucket = "my-bucket";

    ensure_bucket(&*client, bucket)?;
    put_object_bytes(&*client, bucket, "hello.txt", b"Hello, rustfs!", Some("text/plain"))?;

    let data = get_object(&*client, bucket, "hello.txt")?;
    assert_eq!(data, b"Hello, rustfs!");

    Ok(())
}
```

### 工厂注入

```rust
use az_rustfs::client::DefaultS3StorageClientFactory;
use az_rustfs::rustfs::Rustfs;
use az_rustfs::types::RustfsConfig;

let factory = DefaultS3StorageClientFactory::default();
let client = Rustfs::storage_client_with_factory(&factory, RustfsConfig::default());
assert_eq!(std::sync::Arc::strong_count(&client), 1);
```

## 依赖的 crates

- `base64` — Base64 编解码
- `chrono` — 时间戳处理
- `hmac` / `sha2` / `md5` — 签名与哈希
- `quick-xml` — S3 XML 响应解析
- `reqwest` — HTTP 客户端（S3 API 调用）
- `mime_guess` — 根据扩展名推断 Content-Type
- `anyhow` — 错误返回与上下文
