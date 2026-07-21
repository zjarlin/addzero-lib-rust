# az-drive-store

独立网盘的元数据与对象存储抽象。提供统一的 trait 接口，支持 PostgreSQL 正式存储和内存测试实现，对象字节按内容哈希寻址。

## 功能

- `DriveMetadataStore` trait：元数据 CRUD（entry、version、lock、conflict、ignored path、suspended path、sync queue）
- `DriveObjectStore` trait：内容寻址的对象字节存取
- `DriveSyncCoordinator` trait：面向 Git 后端的 sync 协调（prepare / flush），默认提供 `NoopDriveSyncCoordinator`
- `InMemoryDriveMetadataStore`：基于 `Mutex<HashMap>` 的元数据内存实现，适合测试与本地冒烟
- `InMemoryDriveObjectStore`：字节对象的纯内存存储
- 内置子模块 `git_pool`：基于 Git 仓库池的多设备同步存储后端
- 内置子模块 `gitdb_object_store`：基于 gitdb 的分片对象存储
- 完整的数据模型：`DriveEntry`、`DriveVersion`、`DriveLock`、`DriveConflict`、`DriveSyncQueueItem`、`DriveSuspendedPath`、`DriveIgnoredPath`

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-drive-store = { path = "../drive-store" }     # workspace 内部引用
# 或发布后：
# az-drive-store = "0.1"                            # crates.io 引用
```

## 用法

```rust
use az_drive_store::api::{
    DriveMetadataStore, DriveObjectStore, DriveEntryKind,
    InMemoryDriveMetadataStore, InMemoryDriveObjectStore,
    DriveVersion,
};
use az_drive_core::api::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = InMemoryDriveMetadataStore::new();
    let objects = InMemoryDriveObjectStore::new();

    let key = EntryKey::new(
        "main",
        RootAlias::parse("workspace")?,
        RelativePath::parse("docs/readme.md")?,
    );

    let entry = meta.upsert_entry(&key, DriveEntryKind::File).await?;

    let hash = content_hash(b"hello");
    let version = DriveVersion {
        id: Uuid::new_v4(),
        entry_id: entry.id,
        version: 1,
        content_hash: hash.clone(),
        object_key: object_key_for_hash(&hash),
        size_bytes: 5,
        device_id: "device-a".to_owned(),
        modified_at: Utc::now(),
    };
    meta.insert_version(version).await?;

    objects.put_object(&object_key_for_hash(&hash), b"hello").await?;
    let bytes = objects.get_object(&object_key_for_hash(&hash)).await?;
    assert_eq!(bytes, b"hello");

    Ok(())
}
```

## 依赖的 crates

- `az-drive-core` — EntryKey、RelativePath、RootAlias 等基础类型定义
- `az-rustfs` — S3 兼容对象存储抽象（通过 `StorageError` 转换）
- `gitdb` — 本地 gitdb 对象存储（path 依赖）
- `chrono` — 时间戳类型
- `uuid` — 唯一标识符
- `sqlx` — PostgreSQL 驱动（正式实现使用）
- `anyhow` — 错误返回与上下文
