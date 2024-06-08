use ratatui::widgets::ListState;
use std::{io::BufRead, process::Command};

pub enum Action {
    Quit,
    Tick,
    Down,
    Up,
    Home,
    End,
    PageDown,
    PageUp,
    //ToggleFocus,
    //ConfirmAction
    //CancelAction
    //Attach,
    //Kill,
}

pub struct Job {
    pub job_id: String,
    pub array_id: String,
    pub array_step: String,
    pub name: String,
    pub state: String,
    pub state_compact: String,
    pub reason: String,
    pub user: String,
    pub timeused: String,
    pub starttime: String,
    pub submittime: String,
    pub tres: String,
    pub partition: String,
    pub nodelist: String,
}

fn get_jobs() -> Vec<Job> {
    let output_separator = "###";
    let fields = [
        "jobid",
        "name",
        "state",
        "username",
        "timeused",
        "submittime",
        "starttime",
        "tres",
        "partition",
        "nodelist",
        "statecompact",
        "reason",
        "ArrayJobID",
        "ArrayTaskID",
    ];
    let output_format = fields
        .map(|s| s.to_owned() + ":" + output_separator)
        .join(",");

    let jobs: Vec<Job> = Command::new("squeue")
        .arg("--array")
        .arg("--noheader")
        .arg("--Format")
        .arg(&output_format)
        .output()
        .expect("failed to execute squeue")
        .stdout
        .lines()
        .map(|l| l.unwrap().trim().to_string())
        .filter_map(|l| {
            let parts: Vec<_> = l.split(output_separator).collect();

            if parts.len() != fields.len() + 1 {
                return None;
            }

            Some(Job {
                job_id: parts[0].to_owned(),
                name: parts[1].to_owned(),
                state: parts[2].to_owned(),
                user: parts[3].to_owned(),
                timeused: parts[4].to_owned(),
                starttime: parts[5].to_owned(),
                submittime: parts[6].to_owned(),
                tres: parts[7].to_owned(),
                partition: parts[8].to_owned(),
                nodelist: parts[9].to_owned(),
                state_compact: parts[10].to_owned(),
                reason: parts[11].to_owned(),
                array_id: parts[12].to_owned(),
                array_step: parts[13].to_owned(),
            })
        })
        .collect();
    jobs
}

pub struct App {
    pub should_quit: bool,
    pub jobs: Vec<Job>,
    pub state: ListState,
}

impl App {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        App {
            should_quit: false,
            jobs: get_jobs(),
            state,
        }
    }

    pub fn update(&mut self, action: Action) -> () {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Tick => self.jobs = get_jobs(),
            Action::Up => self.previous(),
            Action::Down => self.next(),
            Action::Home => self.home(),
            Action::End => self.end(),
            Action::PageDown => self.down_5(),
            Action::PageUp => self.up_5(),
            _ => (),
        }
    }

    pub fn next(&mut self) -> () {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.jobs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.jobs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn down_5(&mut self) -> () {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.jobs.len() - 5 {
                    self.jobs.len() - 1
                } else {
                    i + 5
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn up_5(&mut self) -> () {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= 5 {
                    0
                } else {
                    i - 5
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn home(&mut self) -> () {
        self.state.select(Some(0))
    }
    pub fn end(&mut self) -> () {
        self.state.select(Some(self.jobs.len() - 1))
    }
}
