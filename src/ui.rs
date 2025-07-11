use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, EditorState, ViewState};
use crate::jobs::{ClusterOverview, Job};

const HELP_SHORT: &str = "q: quit | ?: toggle help | o: toggle overview | <tab>: toggle focus";
const HELP: &str = "lazyslurm is for monitoring SLURM jobs.

## Keymaps

q | Ctrl-c           : quit
?                    : toggle help
o                    : toggle cluster overview
<tab>                : toggle focus
<esc>                : reset view

j | <Down arrow key> : next row
k | <Up arrow key>   : previous row
G | End              : go to last row
g | Home             : go to first row
Ctrl-d | PageDown    : down 5 rows
Ctrl-u | PageUp      : up 5 rows

## Filtering jobs

The live filter box accepts arbitrary regex which
will be matched against all job details. Filtering
also affects the overview panel, so you can do things
like check how many GPUs on a partition are being
used or how many jobs are running with 8 GPUs.

For example:

cj1917               : jobs from user cj1917
loki                 : jobs on node loki
gpu=4                : jobs using 4 GPUs
run.sh               : jobs with run.sh in their name

lory|loki            : jobs on lory OR loki
loki.*gpus=2         : jobs on loki AND with 2 GPUs

NB: using .* for regex AND is order-sensitive
i.e. the first case must match before the second.
The matching order will be the same as the ordering
shown in the Details panel.
";

fn get_short_jobs_list(jobs: &Vec<Job>) -> Vec<ListItem> {
    jobs.iter()
        .map(|j| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<max$.max$} ", j.StateCompact, max = 2),
                    Style::default(),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.JobID, max = 10),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.UserName, max = 10),
                    Style::default().fg(Color::Blue),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.TimeUsed, max = 11),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.NodeList, max = 10),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.Partition, max = 10),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!(" {:<max$.max$}", j.Name, max = 100),
                    Style::default().fg(Color::LightRed),
                ),
            ]))
        })
        .collect()
}

fn style_job_field<'a>(field: String, value: String, max_width: usize) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", { field }, max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(value, Style::default()),
    ])
}

fn get_job_details(job: &Job) -> Paragraph {
    let max_width = 12;
    let fields = Job::field_names();
    let values = Job::field_values(job);

    let lines: Vec<Line> = fields
        .iter()
        .zip(values.iter())
        .map(|(f, v)| {
            style_job_field(
                f.to_owned().to_string(),
                v.to_owned().to_string(),
                max_width,
            )
        })
        .collect();
    let text = Text::from(lines);
    Paragraph::new(text)
}

fn get_user_stats(f: &mut Frame, area: Rect, overview: &ClusterOverview) {
    let header_cells = ["User", "Running", "Pending", "GPUs"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let total_gpus: u32 = overview.user_stats.iter().map(|s| s.gpus_used).sum();

    let total_row = Row::new(vec![
        Cell::from("TOTAL").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(overview.jobs_running.to_string())
            .style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(overview.jobs_pending.to_string())
            .style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(total_gpus.to_string()).style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .height(1)
    .bottom_margin(1);

    let mut rows: Vec<Row> = overview
        .user_stats
        .iter()
        .map(|s| {
            Row::new(vec![
                Cell::from(s.name.as_str()),
                Cell::from(s.running_jobs.to_string()),
                Cell::from(s.pending_jobs.to_string()),
                Cell::from(s.gpus_used.to_string()),
            ])
        })
        .collect();

    rows.insert(0, total_row);

    let table = Table::new(
        rows,
        vec![
            Constraint::Fill(1),
            Constraint::Max(8),
            Constraint::Max(8),
            Constraint::Max(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Overview"),
    );

    f.render_widget(table, area);
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer_layout[1]);

    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");
    let pkg_authors = env!("CARGO_PKG_AUTHORS");
    let pkg_repo = env!("CARGO_PKG_REPOSITORY");
    f.render_widget(
        Paragraph::new(format!(
            "{} v{} | {} | {}",
            pkg_name, pkg_version, pkg_authors, pkg_repo
        )),
        outer_layout[0],
    );

    match app.editor_state {
        EditorState::Editing => {
            f.render_stateful_widget(
                List::new(get_short_jobs_list(&app.jobs))
                    .block(Block::new().borders(Borders::ALL).title_top("Jobs"))
                    .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                    .repeat_highlight_symbol(true),
                inner_layout[0],
                &mut app.list_state,
            );
        }
        EditorState::Normal => {
            f.render_stateful_widget(
                List::new(get_short_jobs_list(&app.jobs))
                    .block(
                        Block::new()
                            .borders(Borders::ALL)
                            .title_top("Jobs")
                            .border_style(Color::Green),
                    )
                    .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                    .repeat_highlight_symbol(true),
                inner_layout[0],
                &mut app.list_state,
            );
        }
    }

    match app.view_state {
        ViewState::Overview => {
            get_user_stats(f, inner_layout[1], &app.overview);
        }
        ViewState::Details => match app.list_state.selected() {
            Some(i) => {
                let job = &app.jobs[i];
                f.render_widget(
                    get_job_details(&job)
                        .block(Block::new().borders(Borders::ALL).title_top("Details")),
                    inner_layout[1],
                );
            }
            None => {
                f.render_widget(
                    Paragraph::new(format!(
                        "No Jobs matching pattern : {}",
                        &app.text_area.lines().concat()
                    ))
                    .block(Block::new().borders(Borders::ALL).title_top("Details")),
                    inner_layout[1],
                );
            }
        },
        ViewState::Help => f.render_widget(
            Paragraph::new(HELP).block(Block::new().borders(Borders::ALL).title_top("Help")),
            inner_layout[1],
        ),
    }

    match app.editor_state {
        EditorState::Normal => {
            app.text_area.set_block(
                Block::new()
                    .borders(Borders::ALL)
                    .title_top("Live filter (regex)"),
            );
            app.text_area.set_cursor_style(Style::default());
        }
        EditorState::Editing => {
            app.text_area.set_block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(Color::Green)
                    .title_top("Live filter (regex)"),
            );
            app.text_area
                .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        }
    }
    f.render_widget(app.text_area.widget(), outer_layout[2]);
    f.render_widget(Paragraph::new(HELP_SHORT), outer_layout[3]);
}



