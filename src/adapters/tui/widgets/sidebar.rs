use crate::adapters::tui::app::App;
use crate::adapters::tui::widgets::colors::Colors;
use chrono::Local;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect, colors: &Colors) {
    let all_tasks = app.task_service.get_all_tasks().unwrap_or_default();
    let total = all_tasks.len();
    let completed = all_tasks.iter().filter(|t| t.completed).count();
    let now = Local::now();

    let stats = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " PROGRESS",
            Style::default().fg(colors.dim_text),
        )]),
        Line::from(vec![
            Span::styled(" Total      ", Style::default().fg(colors.dim_text)),
            Span::styled(total.to_string(), Style::default().fg(colors.primary_text)),
        ]),
        Line::from(vec![
            Span::styled(" Done       ", Style::default().fg(colors.dim_text)),
            Span::styled(
                completed.to_string(),
                Style::default().fg(colors.primary_text),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " STATUS",
            Style::default().fg(colors.dim_text),
        )]),
        Line::from(vec![
            Span::styled(" Date       ", Style::default().fg(colors.dim_text)),
            Span::styled(
                now.format("%Y-%m-%d").to_string(),
                Style::default().fg(colors.primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Time       ", Style::default().fg(colors.dim_text)),
            Span::styled(
                now.format("%H:%M:%S").to_string(),
                Style::default().fg(colors.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled(" OS         ", Style::default().fg(colors.dim_text)),
            Span::styled(
                std::env::consts::OS.to_uppercase(),
                Style::default().fg(colors.primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" DB         ", Style::default().fg(colors.dim_text)),
            Span::styled("SQLITE", Style::default().fg(colors.primary_text)),
            Span::styled(
                if (app.ticks / 10) % 2 == 0 {
                    " ●"
                } else {
                    " ○"
                },
                Style::default().fg(colors.accent),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " METRICS",
            Style::default().fg(colors.dim_text),
        )]),
        Line::from(vec![
            Span::styled(" Latency    ", Style::default().fg(colors.dim_text)),
            Span::styled(
                format!("{}ms", (app.ticks % 15) + 10),
                Style::default().fg(colors.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Entropy    ", Style::default().fg(colors.dim_text)),
            Span::styled(
                format!("0x{:04X}", app.ticks % 0xFFFF),
                Style::default().fg(colors.primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Kernel     ", Style::default().fg(colors.dim_text)),
            Span::styled("L-042", Style::default().fg(colors.primary_text)),
        ]),
    ];
    f.render_widget(Paragraph::new(stats), area);
}
