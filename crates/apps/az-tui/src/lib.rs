//! # az-tui
//!
//! 基于 `ratatui` 和 `crossterm` 的终端 UI 会话与事件基础设施库。
//!
//! 为 AddZero 系列管理后台应用提供统一的 TUI 骨架层，主要包含：
//!
//! - [`TuiSession`] — RAII 终端会话，自动管理原始模式与备用屏幕的进入/退出。
//! - [`EventPump`] — 异步事件泵，将 `crossterm` 键盘事件与定时 Tick 合并为统一的 [`AppEvent`] 流。
//! - [`compute_shell_layout`] — 标准四区域管理壳布局计算（顶栏、侧栏、内容区、底栏）。
//! - [`ShellLayout`] — 四区域布局的矩形信息容器。
//! - [`AzTerminal`] — 类型别名，统一 `Terminal<CrosstermBackend<Stdout>>` 的具体类型。
//!
//! 本 crate 同时 re-export `crossterm` 和 `ratatui`，下游 crate 可直接引用以避免版本冲突。
use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures_util::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use tokio::time::{MissedTickBehavior, interval};

pub use crossterm;
pub use ratatui;

pub type AzTerminal = Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellLayout {
    pub header: Rect,
    pub sidebar: Rect,
    pub content: Rect,
    pub footer: Rect,
}

pub struct TuiSession {
    terminal: AzTerminal,
}

impl TuiSession {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut AzTerminal {
        &mut self.terminal
    }
}

impl Drop for TuiSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

pub struct EventPump {
    events: EventStream,
    ticker: tokio::time::Interval,
}

impl EventPump {
    pub fn new(tick_rate: Duration) -> Self {
        let mut ticker = interval(tick_rate);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self {
            events: EventStream::new(),
            ticker,
        }
    }

    pub async fn next(&mut self) -> io::Result<AppEvent> {
        loop {
            tokio::select! {
                _ = self.ticker.tick() => return Ok(AppEvent::Tick),
                maybe_event = self.events.next() => {
                    match maybe_event {
                        Some(Ok(Event::Key(key))) => return Ok(AppEvent::Key(key)),
                        Some(Ok(Event::Resize(width, height))) => return Ok(AppEvent::Resize(width, height)),
                        Some(Ok(_)) => {}
                        Some(Err(err)) => return Err(err),
                        None => return Ok(AppEvent::Tick),
                    }
                }
            }
        }
    }
}

pub fn compute_shell_layout(area: Rect) -> ShellLayout {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(2),
    ])
    .areas(area);
    let [sidebar, content] =
        Layout::horizontal([Constraint::Length(24), Constraint::Min(20)]).areas(body);
    ShellLayout {
        header,
        sidebar,
        content,
        footer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_layout_keeps_sidebar_width() {
        let area = Rect::new(0, 0, 120, 40);
        let shell = compute_shell_layout(area);
        assert_eq!(shell.sidebar.width, 24);
        assert_eq!(shell.header.height, 3);
        assert_eq!(shell.footer.height, 2);
        assert_eq!(shell.content.width, 96);
    }
}
