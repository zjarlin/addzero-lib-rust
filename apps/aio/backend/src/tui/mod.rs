use std::time::Duration;

use anyhow::Result;
use az_tui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use az_tui::{AppEvent, EventPump, TuiSession};

mod commands;
mod state;
mod view;

use state::{App, Mode, Screen};

pub async fn run_tui() -> Result<()> {
    let mut session = TuiSession::enter()?;
    let mut events = EventPump::new(Duration::from_millis(250));
    let mut app = App::new();
    app.bootstrap().await;

    while !app.should_quit {
        session
            .terminal_mut()
            .draw(|frame| view::render(frame, &app))?;
        match events.next().await? {
            AppEvent::Tick => app.on_tick().await,
            AppEvent::Resize(_, _) => {}
            AppEvent::Key(key) => handle_key(&mut app, key).await,
        }
    }

    Ok(())
}

async fn handle_key(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Char('q')) && matches!(app.mode, Mode::Normal) {
        app.should_quit = true;
        return;
    }

    match app.mode {
        Mode::Normal => handle_normal_mode(app, key).await,
        Mode::SetupEdit => handle_setup_mode(app, key).await,
        Mode::NoteEdit => handle_note_mode(app, key).await,
        Mode::Command => handle_command_mode(app, key).await,
        Mode::TerminalInput => handle_terminal_input_mode(app, key).await,
    }
}

async fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab | KeyCode::Right => {
            app.next_screen();
            if let Err(err) = app.refresh_current().await {
                app.set_error(err);
            }
        }
        KeyCode::BackTab | KeyCode::Left => {
            app.prev_screen();
            if let Err(err) = app.refresh_current().await {
                app.set_error(err);
            }
        }
        KeyCode::Char('r') => {
            if let Err(err) = app.refresh_current().await {
                app.set_error(err);
            }
        }
        KeyCode::Up => match app.current {
            Screen::Notes => {
                app.notes_selected = app.notes_selected.saturating_sub(1);
            }
            Screen::Storage => {
                app.storage_selected = app.storage_selected.saturating_sub(1);
            }
            Screen::Console => {
                app.terminal_selected = app.terminal_selected.saturating_sub(1);
                let _ = app.refresh_terminal_snapshot().await;
            }
            Screen::CliMarket => {
                app.cli_market_selected = app.cli_market_selected.saturating_sub(1);
            }
            Screen::System => app.system.prev_tab(),
            _ => {}
        },
        KeyCode::Down => match app.current {
            Screen::Notes => {
                if app.notes_selected + 1 < app.notes.len() {
                    app.notes_selected += 1;
                }
            }
            Screen::Storage => {
                if app.storage_selected + 1 < app.storage_entries().len() {
                    app.storage_selected += 1;
                }
            }
            Screen::Console => {
                let session_len = app.terminals.as_ref().map_or(0, |list| list.sessions.len());
                if app.terminal_selected + 1 < session_len {
                    app.terminal_selected += 1;
                }
                let _ = app.refresh_terminal_snapshot().await;
            }
            Screen::CliMarket => {
                let entry_len = app
                    .cli_market
                    .as_ref()
                    .map_or(0, |catalog| catalog.entries.len());
                if app.cli_market_selected + 1 < entry_len {
                    app.cli_market_selected += 1;
                }
            }
            Screen::System => app.system.next_tab(),
            _ => {}
        },
        KeyCode::Enter => {
            if app.current == Screen::Storage {
                if let Err(err) = app.open_storage_selected().await {
                    app.set_error(err);
                }
            }
        }
        KeyCode::Backspace => {
            if app.current == Screen::Storage {
                if let Err(err) = app.storage_parent().await {
                    app.set_error(err);
                }
            }
        }
        KeyCode::Char('e') if app.current == Screen::Setup => app.enter_setup_mode(),
        KeyCode::Char('n') if app.current == Screen::Notes => app.enter_note_mode(),
        KeyCode::Char('d') if app.current == Screen::Notes => {
            if let Err(err) = app.delete_selected_note().await {
                app.set_error(err);
            }
        }
        KeyCode::Char(':') if app.current == Screen::System => app.enter_command_mode(),
        KeyCode::Char('c') if app.current == Screen::Console => {
            if let Err(err) = app.create_terminal_session().await {
                app.set_error(err);
            }
        }
        KeyCode::Char('i') if app.current == Screen::Console => app.enter_terminal_input_mode(),
        KeyCode::Char('x') if app.current == Screen::Console => {
            if let Err(err) = app.close_active_terminal().await {
                app.set_error(err);
            }
        }
        KeyCode::Char('s') if app.current == Screen::Skills => {
            if let Err(err) = app.sync_skills().await {
                app.set_error(err);
            }
        }
        _ => {}
    }
}

async fn handle_setup_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_mode(),
        KeyCode::Tab => {
            app.setup_form
                .replace_current(app.setup_buffer.text().to_string());
            app.setup_form.next_field();
            app.setup_buffer
                .set_text(app.setup_form.current_value().to_string());
        }
        KeyCode::BackTab => {
            app.setup_form
                .replace_current(app.setup_buffer.text().to_string());
            app.setup_form.prev_field();
            app.setup_buffer
                .set_text(app.setup_form.current_value().to_string());
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(err) = app.save_setup().await {
                app.set_error(err);
            }
        }
        _ => edit_text_buffer(&mut app.setup_buffer, key, false),
    }
}

async fn handle_note_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_mode(),
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(err) = app.save_note().await {
                app.set_error(err);
            }
        }
        _ => edit_text_buffer(&mut app.note_editor, key, true),
    }
}

async fn handle_command_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_mode(),
        KeyCode::Enter => {
            if let Err(err) = app.submit_command().await {
                app.set_error(err);
            }
        }
        _ => edit_text_buffer(&mut app.command_buffer, key, false),
    }
}

async fn handle_terminal_input_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_mode(),
        KeyCode::Enter => {
            if let Err(err) = app.submit_terminal_input().await {
                app.set_error(err);
            }
        }
        _ => edit_text_buffer(&mut app.terminal_input, key, false),
    }
}

fn edit_text_buffer(buffer: &mut state::TextBuffer, key: KeyEvent, multiline: bool) {
    match key.code {
        KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            buffer.insert_char(ch);
        }
        KeyCode::Enter if multiline => buffer.insert_newline(),
        KeyCode::Backspace => buffer.backspace(),
        KeyCode::Delete => buffer.delete(),
        KeyCode::Left => buffer.move_left(),
        KeyCode::Right => buffer.move_right(),
        KeyCode::Home => buffer.move_home(),
        KeyCode::End => buffer.move_end(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_starts_in_dashboard_normal_mode() {
        let app = App::new();
        assert_eq!(app.current, Screen::Dashboard);
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.status_line, "AIO Rust TUI");
        assert!(!app.should_quit);
    }
}
