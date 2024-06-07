use ratatui::{terminal::Frame, widgets::Paragraph};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    f.render_widget(Paragraph::new("Hello Ratatui! (press 'q' to quit)"), area);
}
