# az-drive-core

独立 AIO 网盘的核心路径规范化、版本模型与冲突决策逻辑。

## 功能

- **路径规范化与校验**：定义 [`RootAlias`]（跨设备逻辑根名称）和 [`RelativePath`]（POSIX 规范化相对路径），拒绝遍历攻击、绝对路径和 Windows 盘符前缀
- **本地↔远程路径映射**：[`RootRegistry`] 将逻辑根别名映射到设备本地绝对路径，通过最长前缀匹配解析主机路径
- **版本与锁模型**：[`EntryVersion`]（UUID + 单调递增版本号 + SHA-256 内容哈希）和 [`LockSnapshot`]（设备持有锁及过期时间）建模远程状态
- **同步决策逻辑**：[`decide_local_change()`] 比较本地基线版本、远程版本、内容哈希和活跃锁，输出 [`ChangeDecision`]（无变化 / 上传新版本 / 冲突 / 被其他设备锁定）
- **内容寻址哈希**：[`content_hash()`] 计算确定性 SHA-256 十六进制摘要；[`object_key_for_hash()`] 生成内容寻址对象存储键
- **冲突文件命名**：[`conflict_file_name()`] 生成确定性的本地冲突副本文件名（如 `report.conflict.mac-book.20260509T123000Z.docx`）
- **保守文本合并**：[`try_safe_text_merge()`] 尝试仅追加模式的安全合并，遇到歧义时返回 `None`
- **CLI 路径展开**：[`expand_path_expression()`] 处理 `~`、`$HOME`、`${HOME}`、`%USERPROFILE%` 展开（仅限本地，不存储为远程标识）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-drive-core = { path = "../az-drive-core" }       # workspace 内部引用
# 或发布后：
# az-drive-core = "0.1"                              # crates.io 引用
```

## 用法

```rust
use az_drive_core::api::{
    ChangeDecision, RootAlias, RootRegistry, content_hash, decide_local_change,
};

fn main() -> anyhow::Result<()> {
// 注册逻辑根并解析本地路径
let mut registry = RootRegistry::default();
registry.add_root(
    RootAlias::parse("home")?,
    std::path::PathBuf::from("/Users/me/Documents"),
)?;

let mapping = registry.resolve_host_path(
    std::path::Path::new("/Users/me/Documents/report.docx"),
    None,
)?;
assert_eq!(mapping.relative_path.as_str(), "report.docx");

// 计算内容哈希
let hash = content_hash(b"hello world");

// 同步决策
let decision = decide_local_change(
    Some(1),
    Some(1),
    &hash,
    Some(&hash),
    None,
    "device-a",
    chrono::Utc::now(),
);
assert_eq!(decision, ChangeDecision::NoopSameContent);
Ok(())
}
```

## 依赖的 crates

- `chrono` - 日期时间处理，用于版本时间戳
- `serde` - 序列化与反序列化
- `sha2` - SHA-256 内容哈希计算
- `anyhow` - 错误链与上下文
- `uuid` - 通用唯一标识符，用于条目和设备标识
