//#![allow(dead_code)]
//#![allow(unused_imports)]
//#![allow(unused_variables)]

mod app;
mod tui;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::{
    io::Result,
    time::{Duration, Instant},
};

use crate::app::{Action, App};
use crate::tui::Tui;

fn main() -> Result<()> {
    let app = App::new();
    let tick_rate = Duration::from_millis(250);
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
                    let action = handle_keys(key);
                    app.update(action);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update(Action::Tick);
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_keys(key: KeyEvent) -> Action {
    match key.code {
        //KeyCode::Left | KeyCode::Char('h') => app.on_left(),
        //KeyCode::Right | KeyCode::Char('l') => app.on_right(),
        KeyCode::Up | KeyCode::Char('k') => Action::Up,
        KeyCode::Down | KeyCode::Char('j') => Action::Down,
        KeyCode::Home | KeyCode::Char('g') => Action::Home,
        KeyCode::End | KeyCode::Char('G') => Action::End,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::PageDown,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::PageUp,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Tab => Action::ToggleView,
        _ => Action::Tick,
    }
}
