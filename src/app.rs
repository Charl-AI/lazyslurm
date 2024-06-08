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

            let id = parts[0];
            let name = parts[1];
            let state = parts[2];
            let user = parts[3];
            let timeused = parts[4];
            let submittime = parts[5];
            let starttime = parts[6];
            let tres = parts[7];
            let partition = parts[8];
            let nodelist = parts[9];
            let state_compact = parts[10];
            let reason = parts[11];

            let array_job_id = parts[12];
            let array_task_id = parts[13];

            Some(Job {
                job_id: id.to_owned(),
                name: name.to_owned(),
                state: state.to_owned(),
                state_compact: state_compact.to_owned(),
                user: user.to_owned(),
                timeused: timeused.to_owned(),
                starttime: starttime.to_owned(),
                submittime: submittime.to_owned(),
                tres: tres.to_owned(),
                partition: partition.to_owned(),
                nodelist: nodelist.to_owned(),
                array_id: array_job_id.to_owned(),
                reason: reason.to_owned(),
                array_step: array_task_id.to_owned(),
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
