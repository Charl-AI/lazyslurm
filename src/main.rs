mod app;
mod jobs;
mod tui;
mod ui;

use app::EditorState;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::{
    io::Result,
    time::{Duration, Instant},
};

use crate::app::{Action, App};
use crate::tui::Tui;

fn main() -> Result<()> {
    let app = App::new();
    let tick_rate = Duration::from_millis(500);
    let mut tui = Tui::new();
    tui.enter();

    run_app(&mut tui, app, tick_rate)?;

    tui.exit();
    Ok(())
}

fn run_app(tui: &mut Tui, mut app: App, tick_rate: Duration) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        tui.terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let action = handle_keys(key, &app);
                    app.update(action);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update(Some(Action::Tick));
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_keys(key: KeyEvent, app: &App) -> Option<Action> {
    match app.editor_state {
        EditorState::Editing => match key.code {
            KeyCode::Esc => Some(Action::ResetView),
            KeyCode::Enter => Some(Action::ToggleFocus),
            KeyCode::Tab => Some(Action::ToggleFocus),
            _ => Some(Action::InputKey(key)),
        },
        EditorState::Normal => match key.code {
            KeyCode::Esc => Some(Action::ResetView),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
            KeyCode::Home | KeyCode::Char('g') => Some(Action::Home),
            KeyCode::End | KeyCode::Char('G') => Some(Action::End),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::PageDown)
            }
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::PageUp)
            }
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Quit)
            }
            KeyCode::Char('?') => Some(Action::ToggleHelp),
            KeyCode::Char('o') => Some(Action::ToggleOverview),
            KeyCode::Tab => Some(Action::ToggleFocus),
            _ => None,
        },
    }
}
