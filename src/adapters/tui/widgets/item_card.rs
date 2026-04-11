use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{adapters::tui::widgets::colors::Colors, domain::task::Task};

pub fn draw_card_item(f: &mut Frame, task: &Task, area: Rect, colors: &Colors, is_selected: bool) {
    let card_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 4,
    };

    let border_style = if is_selected {
        Style::default().fg(colors.accent)
    } else {
        Style::default().fg(colors.dim_text)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .bg(if is_selected {
            colors.card_bg
        } else {
            colors.bg
        });

    f.render_widget(block, card_area);

    let inner_area = card_area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner_area);

    // Top: Title and Description
    let completed = task.is_completed();
    let status_icon = if completed { "⬢" } else { "⬡" };
    let icon_style = if completed {
        Style::default().fg(colors.dim_text)
    } else {
        Style::default().fg(colors.accent)
    };

    let title_style = if completed {
        Style::default()
            .fg(colors.dim_text)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default()
            .fg(colors.primary_text)
            .add_modifier(Modifier::BOLD)
    };

    let top_line = Line::from(vec![
        Span::styled(format!("{} ", status_icon), icon_style),
        Span::styled(task.title(), title_style),
        Span::styled(" - ", Style::default().fg(colors.dim_text)),
        Span::styled(task.description(), Style::default().fg(colors.dim_text)),
    ]);
    f.render_widget(Paragraph::new(top_line), chunks[0]);

    // Bottom: Days counter
    let date_str = match (task.start_date(), task.end_date()) {
        (Some(s), Some(e)) => {
            let today = Local::now().date_naive();
            let start = s.with_timezone(&Local).date_naive();
            let end = e.with_timezone(&Local).date_naive();

            let total_days = (end - start).num_days();
            let elapsed_days = (today - start).num_days().max(0).min(total_days);

            format!(
                "Range: {} -> {} (Day {}/{})",
                s.format("%Y-%m-%d"),
                e.format("%Y-%m-%d"),
                elapsed_days,
                total_days
            )
        }
        (Some(s), None) => {
            let naive_start_date = s.with_timezone(&Local).date_naive();
            let today = Local::now().date_naive();
            let days_since = if today > naive_start_date {
                (today - naive_start_date).num_days()
            } else {
                0
            };

            format!(
                "Started: {} ({} days ago)",
                s.format("%Y-%m-%d"),
                days_since
            )
        }
        (None, Some(e)) => {
            let naive_end = e.with_timezone(&Local).date_naive();
            let today = Local::now().date_naive();
            let days_until = if naive_end > today {
                (naive_end - today).num_days()
            } else {
                0
            };

            format!("Due: {} (in {} days)", e.format("%Y-%m-%d"), days_until)
        }
        (None, None) => "No date set".to_string(),
    };

    let bottom_line = Line::from(vec![Span::styled(
        date_str,
        Style::default()
            .fg(colors.accent)
            .add_modifier(Modifier::ITALIC),
    )]);
    f.render_widget(Paragraph::new(bottom_line), chunks[1]);
}
