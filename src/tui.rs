use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend as Backend;

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
}

impl Tui {
    pub fn new() -> Self {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr())).unwrap();
        Self { terminal }
    }

    pub fn enter(&self) -> () {
        crossterm::terminal::enable_raw_mode().unwrap();
        crossterm::execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )
        .unwrap();
    }

    pub fn exit(&self) -> () {
        crossterm::execute!(
            std::io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )
        .unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}
