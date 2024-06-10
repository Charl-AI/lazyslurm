use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use crate::jobs::{get_jobs, Job};

pub enum Action {
    Quit,
    Tick,
    Down,
    Up,
    Home,
    End,
    PageDown,
    PageUp,
    ToggleHelp,
    ResetView,
    ToggleFocus,
    InputKey(KeyEvent),
}

pub enum ViewState {
    Details,
    Help,
}

pub enum EditorState {
    Normal,
    Editing,
}

pub struct App<'a> {
    pub should_quit: bool,
    pub jobs: Vec<Job>,
    pub list_state: ListState,
    pub view_state: ViewState,
    pub text_area: TextArea<'a>,
    pub editor_state: EditorState,
}

impl App<'_> {
    pub fn new() -> Self {
        let text_area = TextArea::default();
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        App {
            should_quit: false,
            jobs: get_jobs(&"".to_string()),
            list_state,
            view_state: ViewState::Details,
            text_area,
            editor_state: EditorState::Normal,
        }
    }

    pub fn update(&mut self, action: Option<Action>) -> () {
        match action {
            Some(Action::Quit) => self.should_quit = true,
            Some(Action::Tick) => self.jobs = get_jobs(&self.text_area.lines().concat()),
            Some(Action::Up) => self.previous(),
            Some(Action::Down) => self.next(),
            Some(Action::Home) => self.home(),
            Some(Action::End) => self.end(),
            Some(Action::PageDown) => self.down_5(),
            Some(Action::PageUp) => self.up_5(),
            Some(Action::ToggleHelp) => self.toggle_help(),
            Some(Action::ResetView) => self.reset_view(),
            Some(Action::ToggleFocus) => self.toggle_focus(),
            Some(Action::InputKey(key)) => self.text_input(key),
            None => (),
        }
    }

    pub fn next(&mut self) -> () {
        if self.jobs.len() == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.jobs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    pub fn previous(&mut self) {
        if self.jobs.len() == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.jobs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn down_5(&mut self) -> () {
        if self.jobs.len() == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.jobs.len() - 5 {
                    self.jobs.len() - 1
                } else {
                    i + 5
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    pub fn up_5(&mut self) -> () {
        if self.jobs.len() == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i <= 5 {
                    0
                } else {
                    i - 5
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn home(&mut self) -> () {
        if self.jobs.len() == 0 {
            return;
        }
        self.list_state.select(Some(0));
    }
    pub fn end(&mut self) -> () {
        if self.jobs.len() == 0 {
            return;
        }
        self.list_state.select(Some(self.jobs.len() - 1));
    }

    pub fn toggle_help(&mut self) -> () {
        match self.view_state {
            ViewState::Help => self.view_state = ViewState::Details,
            _ => self.view_state = ViewState::Help,
        }
    }

    pub fn reset_view(&mut self) -> () {
        self.view_state = ViewState::Details;
        self.editor_state = EditorState::Normal;
    }

    pub fn toggle_focus(&mut self) -> () {
        match self.editor_state {
            EditorState::Normal => self.editor_state = EditorState::Editing,
            EditorState::Editing => self.editor_state = EditorState::Normal,
        }
    }
    pub fn text_input(&mut self, key: KeyEvent) -> () {
        self.text_area.input(key);
    }
}
