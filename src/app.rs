use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;
use regex::RegexBuilder;
use std::{io::BufRead, process::Command};
use tui_textarea::TextArea;

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

// This macro is a bit crazy.
// the reason we have it is because the names of the fields were
// being repeated in lots of places: the command given to squeue,
// constructing the struct, and using the struct.
// By converting the field names and values to vectors of strings,
// we can shorten the code. NB: this means that the name of each
// field should also be the command given to squeue to retrieve it.
macro_rules! make_field_names_available {
    (struct $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        #[allow(non_snake_case)]
        pub struct $name {
            $(pub $field_name: $field_type,)*
        }

        impl $name {
            pub fn field_names() -> Vec<&'static str> {
                vec![$(stringify!($field_name)),*]
            }
            pub fn field_values(&self) -> Vec<String>{
                vec![
                    $(self.$field_name.clone().to_string()),*
                ]
            }
            fn from_str_parts(parts: Vec<&str>) -> Self {
                let mut iter = parts.into_iter();
                Self {
                    $($field_name: iter.next().unwrap().to_string()),*
                }
            }
        }
    }
}

make_field_names_available!(
    struct Job {
        StateCompact: String,
        State: String,
        Reason: String,
        Name: String,
        UserName: String,
        JobID: String,
        ArrayJobID: String,
        ArrayTaskID: String,
        Partition: String,
        NodeList: String,
        ReqNodes: String,
        SubmitTime: String,
        StartTime: String,
        TimeLimit: String,
        TimeUsed: String,
        TRES: String,
        Priority: String,
        WorkDir: String,
        Command: String,
        STDOUT: String,
        STDERR: String,
    }
);

fn get_jobs(filter_re: &String) -> Vec<Job> {
    let output_separator = "###";
    let fields = Job::field_names().to_owned();
    let output_format: Vec<String> = fields
        .iter()
        .map(|s| s.to_string().to_owned() + ":" + output_separator)
        .collect();
    let format_str: String = output_format.join(",");

    let re = RegexBuilder::new(filter_re)
        .case_insensitive(true)
        .build()
        // if invalid regex, just use ""
        .unwrap_or(RegexBuilder::new("").build().unwrap());

    let jobs: Vec<Job> = Command::new("squeue")
        .arg("--array")
        .arg("--noheader")
        .arg("--Format")
        .arg(format_str)
        .output()
        .expect("failed to execute squeue")
        .stdout
        .lines()
        .map(|l| l.unwrap().trim().to_string())
        .filter_map(|l| {
            if !re.is_match(l.as_str()) {
                return None;
            }
            let parts: Vec<_> = l.split(output_separator).collect();
            if parts.len() != fields.len() + 1 {
                return None;
            }
            let job = Job::from_str_parts(parts);
            return Some(job);
        })
        .collect();
    jobs
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
