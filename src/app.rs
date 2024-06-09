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
    ToggleView,
    ToggleHelp,
    ResetView,
}

pub enum View {
    Details,
    Help,
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
    pub timelimit: String,
    pub tres: String,
    pub partition: String,
    pub nodelist: String,
    pub priority: String,
    pub workdir: String,
    pub command: String,
}

fn get_jobs(my_jobs_only: bool, my_user: &String) -> Vec<Job> {
    let output_separator = "###";
    let fields = [
        "jobid",
        "name",
        "state",
        "username",
        "timeused",
        "submittime",
        "starttime",
        "timelimit",
        "tres",
        "partition",
        "nodelist",
        "statecompact",
        "reason",
        "ArrayJobID",
        "ArrayTaskID",
        "priority",
        "workdir",
        "command",
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

            if my_jobs_only {
                if parts[3].to_owned() != my_user.to_owned() {
                    return None;
                }
            }

            if parts.len() != fields.len() + 1 {
                return None;
            }

            Some(Job {
                job_id: parts[0].to_owned(),
                name: parts[1].to_owned(),
                state: parts[2].to_owned(),
                user: parts[3].to_owned(),
                timeused: parts[4].to_owned(),
                submittime: parts[5].to_owned(),
                starttime: parts[6].to_owned(),
                timelimit: parts[7].to_owned(),
                tres: parts[8].to_owned(),
                partition: parts[9].to_owned(),
                nodelist: parts[10].to_owned(),
                state_compact: parts[11].to_owned(),
                reason: parts[12].to_owned(),
                array_id: parts[13].to_owned(),
                array_step: parts[14].to_owned(),
                priority: parts[15].to_owned(),
                workdir: parts[16].to_owned(),
                command: parts[17].to_owned(),
            })
        })
        .collect();
    jobs
}

pub struct App {
    pub should_quit: bool,
    pub jobs: Vec<Job>,
    pub list_state: ListState,
    pub view_state: View,
    pub my_jobs_only: bool,
    pub my_user: String,
}

impl App {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        let my_user = String::from_utf8(
            Command::new("whoami")
                .output()
                .expect("failed to execute whoami")
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        App {
            should_quit: false,
            jobs: get_jobs(false, &my_user),
            list_state: state,
            view_state: View::Details,
            my_jobs_only: false,
            my_user,
        }
    }

    pub fn update(&mut self, action: Option<Action>) -> () {
        match action {
            Some(Action::Quit) => self.should_quit = true,
            Some(Action::Tick) => self.jobs = get_jobs(self.my_jobs_only, &self.my_user),
            Some(Action::Up) => self.previous(),
            Some(Action::Down) => self.next(),
            Some(Action::Home) => self.home(),
            Some(Action::End) => self.end(),
            Some(Action::PageDown) => self.down_5(),
            Some(Action::PageUp) => self.up_5(),
            Some(Action::ToggleView) => self.toggle_job_view(),
            Some(Action::ToggleHelp) => self.toggle_help(),
            Some(Action::ResetView) => self.reset_view(),
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

    pub fn toggle_job_view(&mut self) -> () {
        if self.my_jobs_only == false {
            self.my_jobs_only = true;
            self.list_state.select(Some(0));
        } else {
            self.my_jobs_only = false;
            self.list_state.select(Some(0));
        }
    }
    pub fn toggle_help(&mut self) -> () {
        match self.view_state {
            View::Help => self.view_state = View::Details,
            _ => self.view_state = View::Help,
        }
    }

    pub fn reset_view(&mut self) -> () {
        self.view_state = View::Details
    }
}
