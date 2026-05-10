# az-tui

基于 `ratatui` 和 `crossterm` 的终端 UI 会话与事件基础设施库，为 AddZero 系列管理后台应用提供统一的 TUI 骨架层。

## 功能

- **TuiSession** — RAII 终端会话，自动管理原始模式（raw mode）与备用屏幕（alternate screen）的进入和退出
- **EventPump** — 异步事件泵，将键盘事件与定时 Tick 合并为统一的 `AppEvent` 流
- **compute_shell_layout** — 标准四区域管理壳布局计算（顶栏、侧栏、内容区、底栏）
- **ShellLayout** — 四区域布局的矩形信息容器
- **AzTerminal** — `Terminal<CrosstermBackend<Stdout>>` 类型别名，统一终端类型
- re-export `crossterm` 和 `ratatui`，避免下游版本冲突

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-tui = { path = "../az-tui" }       # workspace 内部引用
# 或发布后：
# az-tui = "0.1"                      # crates.io 引用
```

## 用法

```rust
use az_tui::{TuiSession, EventPump, AppEvent, compute_shell_layout};
use std::time::Duration;

// 初始化终端会话
let mut session = TuiSession::enter()?;
let terminal = session.terminal_mut();

// 创建异步事件泵
let mut pump = EventPump::new(Duration::from_millis(16));

// 事件循环
loop {
    match pump.next().await? {
        AppEvent::Tick => { /* 刷新界面 */ }
        AppEvent::Key(key) => { /* 处理按键 */ break; }
        AppEvent::Resize(w, h) => { /* 处理窗口尺寸变化 */ }
    }
}

// 计算标准布局
let area = terminal.size()?;
let layout = compute_shell_layout(area);
// layout.header, layout.sidebar, layout.content, layout.footer
```

## 依赖的 crates

- `crossterm` — 跨平台终端 I/O 与事件处理
- `ratatui` — 终端 UI 渲染框架
- `tokio` — 异步运行时（定时 Tick）
- `futures-util` — 异步 Stream 扩展
