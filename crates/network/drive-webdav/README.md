# az-drive-webdav

面向独立网盘的极简 WebDAV 适配层，通过 Axum 暴露 WebDAV 标准方法。

## 功能

- 通过 Axum HTTP 服务器暴露 WebDAV 方法（PROPFIND、MKCOL、GET、PUT、DELETE、MOVE、COPY、LOCK、UNLOCK）
- 将身份管理和版本控制委托给共享的网盘元数据存储
- 将文件对象存取委托给对象存储后端
- 支持按空间（space）和根别名（root alias）组织文件目录
- 支持 WebDAV 锁定机制（LOCK/UNLOCK）及锁令牌校验
- 通过设备 ID 和所有者 HTTP 头实现多设备/多用户区分

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-drive-webdav = { path = "../drive-webdav" }  # workspace 内部引用
# 或发布后：
# az-drive-webdav = "0.1"                           # crates.io 引用
```

## 用法

```rust,no_run
use az_drive_store::api::{DriveMetadataStore, DriveObjectStore};
use az_drive_webdav::api::{DriveWebdavState, drive_webdav_router};
use std::sync::Arc;

# fn my_metadata_store() -> Arc<dyn DriveMetadataStore> {
#     todo!("provide metadata store")
# }
# fn my_object_store() -> Arc<dyn DriveObjectStore> {
#     todo!("provide object store")
# }

// 构造 WebDAV 路由（需要已实现的元数据存储和对象存储）
let state = DriveWebdavState::new(
    my_metadata_store(),
    my_object_store(),
);
let router = drive_webdav_router(state);
// 将 router 挂载到 Axum 服务器即可使用
```

## 依赖的 crates

- `az-drive-core` - 网盘核心类型（路径、哈希、键值）
- `az-drive-store` - 网盘存储抽象（元数据存储、对象存储、版本控制）
- `axum` - HTTP 框架，提供路由和请求处理
- `chrono` - 时间处理（锁过期等）
- `serde` - 序列化支持
- `anyhow` - 错误上下文与统一 `Result`
- `uuid` - 锁令牌生成
