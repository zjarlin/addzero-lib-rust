# az-config-center-contract

AIO Desktop 配置中心契约层，定义 Shell 组件注册表、桌面后端状态查询、组件构建等全部共享数据类型。

## 功能

- Shell 组件类型枚举（`ShellComponentKind`）：Export、Alias、Function、Snippet 四种
- Shell 组件完整数据模型（`ShellComponent`）及其增删改查 DTO
- 组件注册表快照（`ShellComponentRegistry`）、构建配置与构建结果
- 桌面后端健康状态（`DesktopBackendStatus`）
- 公共常量：默认输出路径（`DEFAULT_SHELL_OUTPUT_PATH`）、桌面令牌 header 名（`DESKTOP_SESSION_TOKEN_HEADER`）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-config-center-contract = { path = "../az-config-center-contract" }  # workspace 内部引用
# 或发布后：
# az-config-center-contract = "0.1"                                      # crates.io 引用
```

## 用法

```rust
use az_config_center_contract::{
    ShellComponent, ShellComponentKind, ShellComponentUpsert, ShellComponentPatch,
    ShellComponentRemove, ShellComponentRegistry, DesktopBackendStatus,
};

// 构造一个 Shell 组件
let component = ShellComponent {
    name: "grep-log".into(),
    kind: ShellComponentKind::Function,
    summary: "搜索应用日志".into(),
    body: Some("grep -rn \"$1\" /var/log/app/*.log".into()),
    ..Default::default()
};

// 构造 upsert 请求
let upsert = ShellComponentUpsert {
    name: component.name.clone(),
    kind: component.kind,
    summary: component.summary,
    body: component.body,
    ..Default::default()
};

// 构造 patch 请求（仅修改摘要）
let patch = ShellComponentPatch {
    name: "grep-log".into(),
    summary: Some("按关键字搜索应用日志".into()),
    ..Default::default()
};
```

## 依赖的 crates

- `serde` — 所有结构体的序列化与反序列化