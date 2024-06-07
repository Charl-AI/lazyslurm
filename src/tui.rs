use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend as Backend;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

pub type Frame<'a> = ratatui::Frame<'a, Backend<std::io::Stderr>>;

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        Ok(())
    }

    pub fn exit(&self) -> Result<()> {
        crossterm::execute!(
            std::io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn suspend(&self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&self) -> Result<()> {
        self.enter()?;
        Ok(())
    }
}
