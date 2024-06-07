pub enum Action {
    Quit,
    Tick,
    Down,
    Up,
    //ToggleFocus,
    //Attach,
    //Kill,
}

pub struct App {
    pub should_quit: bool,
    //jobs: Vec<Job>,
}

impl App {
    pub fn update(&mut self, action: Action) -> () {
        match action {
            Action::Quit => self.should_quit = true,
            _ => (),
        }
    }
}
