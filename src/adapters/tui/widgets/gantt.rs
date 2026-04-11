use crate::adapters::tui::app::App;
use crate::adapters::tui::widgets::colors::Colors;
use chrono::{Duration, Utc};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw_gantt_chart(f: &mut Frame, app: &mut App, area: Rect, colors: &Colors) {
    let tasks = app.get_filtered_items();
    let tasks_with_dates: Vec<_> = tasks
        .iter()
        .filter(|t| t.start_date.is_some() || t.end_date.is_some())
        .collect();

    if tasks_with_dates.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Gantt Chart ")
            .border_style(Style::default().fg(colors.dim_text));
        let message = Line::from(
            "No tasks with dates found. Add tasks with start/end dates to see them here.",
        );
        f.render_widget(List::new(vec![ListItem::new(message)]).block(block), area);
        return;
    }

    // Find overall date range
    let mut min_date = tasks_with_dates
        .iter()
        .filter_map(|t| t.start_date.or(t.end_date))
        .min()
        .unwrap_or_else(Utc::now);

    let mut max_date = tasks_with_dates
        .iter()
        .filter_map(|t| t.end_date.or(t.start_date))
        .max()
        .unwrap_or_else(Utc::now);

    // Add some padding to dates if they are the same
    if min_date == max_date {
        min_date = min_date - Duration::days(1);
        max_date = max_date + Duration::days(7);
    } else {
        // Just a little padding to make it look nicer
        min_date = min_date - Duration::days(1);
        max_date = max_date + Duration::days(1);
    }

    let total_days = (max_date - min_date).num_days().max(1);

    // Width for task labels
    let label_width = 25;
    let chart_width = area.width.saturating_sub(label_width as u16 + 10);

    if chart_width < 5 {
        // Not enough space for the chart
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Gantt Chart ")
            .border_style(Style::default().fg(colors.dim_text));
        let message = Line::from("Not enough horizontal space to render Gantt chart.");
        f.render_widget(List::new(vec![ListItem::new(message)]).block(block), area);
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();

    // Timeline Header
    let mut timeline_spans = vec![
        Span::styled(
            format!("{: <width$}", "Timeline", width = label_width),
            Style::default()
                .fg(colors.dim_text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ "),
    ];

    // Add some date markers to the timeline
    let num_markers = 5.min(chart_width / 10);
    if num_markers > 0 {
        let mut last_pos = 0;
        for i in 0..num_markers {
            let day_offset =
                (i as f64 * total_days as f64 / (num_markers - 1).max(1) as f64) as i64;
            let date = min_date + Duration::days(day_offset);
            let pos = (day_offset as f64 / total_days as f64 * chart_width as f64) as u16;

            let date_str = date.format("%m-%d").to_string();
            let padding = pos.saturating_sub(last_pos).saturating_sub(1);
            for _ in 0..padding {
                timeline_spans.push(Span::raw(" "));
            }
            timeline_spans.push(Span::styled(date_str, Style::default().fg(colors.dim_text)));
            last_pos = pos + 4; // Length of MM-DD
        }
    }
    items.push(ListItem::new(Line::from(timeline_spans)));
    items.push(ListItem::new(Line::from(vec![
        Span::raw("─".repeat(label_width)),
        Span::raw("─┼─"),
        Span::raw("─".repeat(chart_width as usize)),
    ])));

    for item in tasks {
        let mut label = item.title.clone();
        if label.len() > label_width {
            label.truncate(label_width - 3);
            label.push_str("...");
        }
        let label_span = Span::styled(
            format!("{: <width$}", label, width = label_width),
            Style::default().fg(if item.completed {
                colors.dim_text
            } else {
                colors.primary_text
            }),
        );

        let mut spans = vec![label_span, Span::raw(" │ ")];

        match (item.start_date, item.end_date) {
            (Some(start), Some(end)) => {
                let start_offset = (start - min_date).num_days().max(0);
                let duration = (end - start).num_days().max(1);

                let start_pos =
                    (start_offset as f64 / total_days as f64 * chart_width as f64) as u16;
                let bar_width = (duration as f64 / total_days as f64 * chart_width as f64) as u16;
                let bar_width = bar_width.max(1);

                for _ in 0..start_pos {
                    spans.push(Span::raw(" "));
                }

                let bar_color = if item.completed {
                    colors.dim_text
                } else if item.important {
                    colors.alert
                } else {
                    colors.accent
                };
                spans.push(Span::styled(
                    "█".repeat(bar_width as usize),
                    Style::default().fg(bar_color),
                ));
            }
            (Some(start), None) => {
                let start_offset = (start - min_date).num_days().max(0);
                let start_pos =
                    (start_offset as f64 / total_days as f64 * chart_width as f64) as u16;
                for _ in 0..start_pos {
                    spans.push(Span::raw(" "));
                }
                spans.push(Span::styled("▶", Style::default().fg(colors.accent)));
                spans.push(Span::styled("┄", Style::default().fg(colors.dim_text)));
            }
            (None, Some(end)) => {
                let end_offset = (end - min_date).num_days().max(0);
                let end_pos = (end_offset as f64 / total_days as f64 * chart_width as f64) as u16;
                spans.push(Span::styled(
                    "┄".repeat(end_pos as usize),
                    Style::default().fg(colors.dim_text),
                ));
                spans.push(Span::styled("◀", Style::default().fg(colors.accent)));
            }
            _ => {
                // No dates, just some indicators or nothing
            }
        }

        items.push(ListItem::new(Line::from(spans)));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Gantt Chart: {} to {} ",
                    min_date.format("%Y-%m-%d"),
                    max_date.format("%Y-%m-%d")
                ))
                .border_style(Style::default().fg(colors.dim_text)),
        )
        .highlight_style(
            Style::default()
                .bg(colors.card_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("→ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}
