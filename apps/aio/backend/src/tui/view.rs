use az_tui::compute_shell_layout;
use az_tui::ratatui::Frame;
use az_tui::ratatui::layout::{Alignment, Constraint, Layout, Rect};
use az_tui::ratatui::prelude::Stylize;
use az_tui::ratatui::style::{Color, Modifier, Style};
use az_tui::ratatui::text::{Line, Text};
use az_tui::ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table, Tabs, Wrap,
};

use super::state::{App, Mode, Screen, SetupForm};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let shell = compute_shell_layout(frame.area());

    let title = Paragraph::new(Line::from(vec![
        "AIO ".into(),
        "Rust TUI".bold().fg(Color::Cyan),
        format!("  [{}]", app.current.title()).into(),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Workspace"));
    frame.render_widget(title, shell.header);

    let sidebar_items = Screen::ALL
        .into_iter()
        .map(|screen| ListItem::new(screen.title()))
        .collect::<Vec<_>>();
    let mut sidebar_state = ListState::default();
    sidebar_state.select(Screen::ALL.iter().position(|screen| *screen == app.current));
    let sidebar = List::new(sidebar_items)
        .block(Block::default().borders(Borders::ALL).title("Modules"))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black))
        .highlight_symbol(">");
    frame.render_stateful_widget(sidebar, shell.sidebar, &mut sidebar_state);

    render_content(frame, shell.content, app);

    let footer_text = app
        .last_error
        .as_ref()
        .map(|message| Line::styled(message.clone(), Style::default().fg(Color::Red)))
        .unwrap_or_else(|| Line::from(format!("{} | {}", app.current.help(), app.status_line)));
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(mode_title(app.mode)),
        );
    frame.render_widget(footer, shell.footer);

    render_overlay(frame, app);
}

fn render_content(frame: &mut Frame<'_>, area: Rect, app: &App) {
    match app.current {
        Screen::Dashboard => render_dashboard(frame, area, app),
        Screen::Setup => render_setup(frame, area, app),
        Screen::Assets => render_assets(frame, area, app),
        Screen::Knowledge => render_knowledge(frame, area, app),
        Screen::Notes => render_notes(frame, area, app),
        Screen::Storage => render_storage(frame, area, app),
        Screen::Console => render_console(frame, area, app),
        Screen::Skills => render_skills(frame, area, app),
        Screen::System => render_system(frame, area, app),
        Screen::CliMarket => render_cli_market(frame, area, app),
        Screen::Cloudflare => render_cloudflare(frame, area, app),
    }
}

fn render_dashboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let items = app
        .dashboard_lines()
        .into_iter()
        .map(ListItem::new)
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Summary")),
        area,
    );
}

fn render_setup(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [top, bottom] = Layout::vertical([Constraint::Length(8), Constraint::Min(8)]).areas(area);
    let mut lines = Vec::new();
    if let Some(bootstrap) = &app.bootstrap {
        lines.push(Line::from(format!(
            "setup_required: {}",
            bootstrap.setup_required
        )));
        lines.push(Line::from(format!("message: {}", bootstrap.message)));
        lines.push(Line::from(format!(
            "config_path: {}",
            bootstrap.config_path
        )));
    }
    if let Some(platform) = &app.platform {
        lines.push(Line::from(format!(
            "postgres reachable: {} | minio reachable: {}",
            platform.postgres.reachable, platform.minio.reachable
        )));
        lines.push(Line::from(format!("bucket: {}", platform.minio.bucket)));
    }
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("Environment")),
        top,
    );

    let rows = SetupForm::labels()
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let value = match index {
                0 => app.setup_form.database_url.as_str(),
                1 => app.setup_form.minio_endpoint.as_str(),
                2 => app.setup_form.minio_access_key.as_str(),
                3 => mask_secret(app.setup_form.minio_secret_key.as_str()),
                _ => app.setup_form.minio_region.as_str(),
            };
            Row::new([label.to_string(), value.to_string()])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Table::new(rows, [Constraint::Length(20), Constraint::Min(20)])
            .block(Block::default().borders(Borders::ALL).title("Setup Wizard")),
        bottom,
    );
}

fn render_assets(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let text = if let Some(graph) = &app.asset_graph {
        format!(
            "items={} edges={} tags={}\nnotes={} software={} packages={}\nwarnings={}",
            graph.items.len(),
            graph.edges.len(),
            graph.tags.len(),
            graph.note_count,
            graph.software_count,
            graph.package_count,
            graph.warnings.join(" | ")
        )
    } else {
        "按 r 加载资产图。".to_string()
    };
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Asset Graph"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_knowledge(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [top, bottom] = Layout::vertical([Constraint::Length(8), Constraint::Min(8)]).areas(area);
    let summary = if let Some(feed) = &app.knowledge_feed {
        format!(
            "total={} open_exceptions={}\nwarnings={}\n{}",
            feed.total,
            feed.open_exception_count,
            feed.warnings.join(" | "),
            feed.items
                .iter()
                .take(6)
                .map(|item| format!("- {} [{}]", item.title, item.status))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        "按 r 加载知识图谱摘要。".to_string()
    };
    frame.render_widget(
        Paragraph::new(summary)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Knowledge Feed"),
            )
            .wrap(Wrap { trim: false }),
        top,
    );
    let exceptions = app
        .knowledge_exceptions
        .iter()
        .map(|item| ListItem::new(format!("{} :: {}", item.subject_title, item.reason)))
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(exceptions).block(Block::default().borders(Borders::ALL).title("Exceptions")),
        bottom,
    );
}

fn render_notes(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [left, right] =
        Layout::horizontal([Constraint::Length(36), Constraint::Min(20)]).areas(area);
    let items = app
        .notes
        .iter()
        .map(|note| ListItem::new(format!("{}  {}", note.title, note.updated_at)))
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select((!app.notes.is_empty()).then_some(app.notes_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Notes"))
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
        left,
        &mut state,
    );
    let detail = app
        .active_note()
        .map(|note| format!("{}\n\n{}", note.relative_path, note.body))
        .unwrap_or_else(|| "还没有笔记，按 n 新建。".to_string());
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().borders(Borders::ALL).title("Detail"))
            .wrap(Wrap { trim: false }),
        right,
    );
}

fn render_storage(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [left, right] =
        Layout::horizontal([Constraint::Length(40), Constraint::Min(20)]).areas(area);
    let items = app
        .storage_entries()
        .into_iter()
        .map(|(label, is_folder)| {
            let prefix = if is_folder { "[D]" } else { "[F]" };
            ListItem::new(format!("{prefix} {label}"))
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select((!items.is_empty()).then_some(app.storage_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Browse"))
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
        left,
        &mut state,
    );
    let detail = app.storage.as_ref().map_or_else(
        || "按 r 加载对象存储。".to_string(),
        |storage| {
            format!(
                "bucket={}\nprefix={}\nfolders={}\nfiles={}\nbackend={}",
                storage.bucket,
                storage.current_prefix,
                storage.folder_count,
                storage.file_count,
                storage.backend_label
            )
        },
    );
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().borders(Borders::ALL).title("Storage"))
            .wrap(Wrap { trim: false }),
        right,
    );
}

fn render_console(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [left, right] =
        Layout::horizontal([Constraint::Length(36), Constraint::Min(20)]).areas(area);
    let items = app
        .terminals
        .as_ref()
        .map(|list| {
            list.sessions
                .iter()
                .map(|session| {
                    ListItem::new(format!(
                        "{} [{}] {}",
                        session.title,
                        session.profile.label(),
                        session.state_code()
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut state = ListState::default();
    state.select((!items.is_empty()).then_some(app.terminal_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Sessions"))
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
        left,
        &mut state,
    );
    let detail = app
        .terminal_snapshot
        .as_ref()
        .map(|snapshot| snapshot.screen.clone())
        .unwrap_or_else(|| "按 c 创建 Shell 会话。".to_string());
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().borders(Borders::ALL).title("Screen"))
            .wrap(Wrap { trim: false }),
        right,
    );
}

fn render_skills(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [top, bottom] = Layout::vertical([Constraint::Length(6), Constraint::Min(8)]).areas(area);
    frame.render_widget(
        Paragraph::new(app.skills_summary())
            .block(Block::default().borders(Borders::ALL).title("Sync Status"))
            .wrap(Wrap { trim: false }),
        top,
    );
    let items = app
        .skills
        .iter()
        .map(|skill| ListItem::new(format!("{} :: {}", skill.name, skill.description)))
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Skills")),
        bottom,
    );
}

fn render_system(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [tabs_area, body, output] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(10),
    ])
    .areas(area);
    let tab_titles = super::state::SystemSnapshot::tab_titles()
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
    frame.render_widget(
        Tabs::new(tab_titles)
            .select(app.system.tab)
            .block(Block::default().borders(Borders::ALL).title("System Tabs"))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        tabs_area,
    );
    let items = app
        .system_tab_lines()
        .into_iter()
        .map(ListItem::new)
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Data")),
        body,
    );
    frame.render_widget(
        Paragraph::new(app.system_output.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Output"),
            )
            .wrap(Wrap { trim: false }),
        output,
    );
}

fn render_cli_market(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [left, right] =
        Layout::horizontal([Constraint::Length(42), Constraint::Min(20)]).areas(area);
    let items = app
        .cli_market
        .as_ref()
        .map(|catalog| {
            catalog
                .entries
                .iter()
                .map(|entry| ListItem::new(format!("{} {}", entry.slug, entry.latest_version)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut state = ListState::default();
    state.select((!items.is_empty()).then_some(app.cli_market_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("CLI Entries"))
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
        left,
        &mut state,
    );
    frame.render_widget(
        Paragraph::new(app.cli_market_detail())
            .block(Block::default().borders(Borders::ALL).title("Entry Detail"))
            .wrap(Wrap { trim: false }),
        right,
    );
}

fn render_cloudflare(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [top, bottom] = Layout::vertical([Constraint::Length(8), Constraint::Min(8)]).areas(area);
    let summary = app.cloudflare.as_ref().map_or_else(
        || "按 r 加载 Cloudflare Tunnel 状态。".to_string(),
        |status| {
            format!(
                "config_exists={} tunnel_running={}\nhosts={} http={} tcp={}\nconfig={}",
                status.config_exists,
                status.tunnel_running,
                status.host_count,
                status.http_count,
                status.tcp_count,
                status.config_path
            )
        },
    );
    frame.render_widget(
        Paragraph::new(summary)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tunnel Summary"),
            )
            .wrap(Wrap { trim: false }),
        top,
    );
    let items = app
        .cloudflare
        .as_ref()
        .map(|status| {
            status
                .hosts
                .iter()
                .map(|host| ListItem::new(format!("{} -> {}", host.hostname, host.service)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Ingress Hosts"),
        ),
        bottom,
    );
}

fn render_overlay(frame: &mut Frame<'_>, app: &App) {
    match app.mode {
        Mode::Normal => {}
        Mode::SetupEdit => render_text_popup(
            frame,
            "Edit Setup Field",
            app.setup_buffer.text(),
            app.setup_buffer.cursor_line_col(),
        ),
        Mode::NoteEdit => render_text_popup(
            frame,
            "New Note",
            app.note_editor.text(),
            app.note_editor.cursor_line_col(),
        ),
        Mode::Command => render_text_popup(
            frame,
            "Command",
            app.command_buffer.text(),
            app.command_buffer.cursor_line_col(),
        ),
        Mode::TerminalInput => render_text_popup(
            frame,
            "Terminal Input",
            app.terminal_input.text(),
            app.terminal_input.cursor_line_col(),
        ),
    }
}

fn render_text_popup(frame: &mut Frame<'_>, title: &str, value: &str, cursor: (u16, u16)) {
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(value)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false }),
        area,
    );
    let cursor_x = area.x.saturating_add(1).saturating_add(cursor.1);
    let cursor_y = area.y.saturating_add(1).saturating_add(cursor.0);
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let [vertical] = Layout::vertical([Constraint::Percentage(height_percent)]).areas(area);
    let [horizontal] = Layout::horizontal([Constraint::Percentage(width_percent)]).areas(vertical);
    horizontal
}

fn mode_title(mode: Mode) -> &'static str {
    match mode {
        Mode::Normal => "Normal",
        Mode::SetupEdit => "Setup Edit",
        Mode::NoteEdit => "Note Edit",
        Mode::Command => "Command",
        Mode::TerminalInput => "Terminal Input",
    }
}

fn mask_secret(secret: &str) -> &str {
    if secret.is_empty() { "" } else { "********" }
}

trait TerminalStateLabel {
    fn state_code(&self) -> &'static str;
}

impl TerminalStateLabel for crate::services::TerminalSessionSummaryDto {
    fn state_code(&self) -> &'static str {
        match self.state {
            crate::services::TerminalSessionStateDto::Running => "running",
            crate::services::TerminalSessionStateDto::Exited => "exited",
            crate::services::TerminalSessionStateDto::Failed => "failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use az_tui::ratatui::Terminal;
    use az_tui::ratatui::backend::TestBackend;

    use super::*;
    use crate::tui::state::App;

    #[test]
    fn dashboard_smoke_renders_tui_shell() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut app = App::new();
        app.status_line = "ready".to_string();
        terminal
            .draw(|frame| render(frame, &app))
            .expect("draw dashboard");
        let buffer = terminal.backend().buffer();
        let content = buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("Rust TUI"));
        assert!(content.contains("Modules"));
        assert!(content.contains("Summary"));
    }
}
