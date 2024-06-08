use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, Job};

fn get_short_jobs_list(jobs: &Vec<Job>) -> Vec<ListItem> {
    jobs.iter()
        .map(|j| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<max$.max$} ", j.state_compact, max = 2),
                    Style::default(),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.job_id, max = 6),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.partition, max = 11),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.user, max = 6),
                    Style::default().fg(Color::Blue),
                ),
                Span::styled(
                    format!(" {:<max$.max$} ", j.timeused, max = 11),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" {:<max$.max$}", j.name, max = 100),
                    Style::default().fg(Color::LightRed),
                ),
            ]))
        })
        .collect()
}

fn get_job_details(job: &Job) -> Paragraph {
    let max_width = 11;
    let status = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "Status", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{} ({})", job.state, job.state_compact),
            Style::default(),
        ),
    ]);
    let user = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "User", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.user, Style::default()),
    ]);
    let reason = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "Reason", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.reason, Style::default()),
    ]);
    let jobid = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "JobID", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.job_id, Style::default()),
    ]);
    let arrayid = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "ArrayID", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.array_id, Style::default()),
    ]);
    let array_step = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "ArrayStep", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.array_step, Style::default()),
    ]);
    let partition = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "Partition", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.partition, Style::default()),
    ]);
    let nodelist = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "NodeList", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.nodelist, Style::default()),
    ]);
    let submittime = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "SubmitTime", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.submittime, Style::default()),
    ]);
    let starttime = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "StartTime", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.starttime, Style::default()),
    ]);
    let timeused = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "RunningTime", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.timeused, Style::default()),
    ]);
    let tres = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "TRES", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.tres, Style::default()),
    ]);
    let command = Line::from(vec![
        Span::styled(
            format!("{:<max$.max$}", "Command", max = max_width),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(&job.name, Style::default()),
    ]);

    let text = Text::from(vec![
        status, user, reason, jobid, arrayid, array_step, partition, nodelist, submittime,
        starttime, timeused, tres, command,
    ]);

    Paragraph::new(text)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(f.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer_layout[1]);

    f.render_widget(
        Paragraph::new("LazySLURM v0.0 | C Jones | github.com/Charl-AI/lazyslurm"),
        outer_layout[0],
    );
    f.render_stateful_widget(
        List::new(get_short_jobs_list(&app.jobs))
            .block(Block::new().borders(Borders::ALL).title_top("Jobs"))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .repeat_highlight_symbol(true),
        inner_layout[0],
        &mut app.state,
    );
    let idx = app.state.selected().unwrap();
    let job = &app.jobs[idx];
    f.render_widget(
        get_job_details(&job).block(Block::new().borders(Borders::ALL).title_top("Details")),
        inner_layout[1],
    );
    f.render_widget(
        Paragraph::new(
            "q: quit | k: up | j: down | a: attach to job | x: cancel job | <tab>: show only my jobs (toggle)",
        ),
        outer_layout[2],
    );
}
