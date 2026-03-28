use crate::adapters::tui::app::App;
use crate::adapters::tui::widgets::colors::Colors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw_task_list(f: &mut Frame, app: &mut App, area: Rect, colors: &Colors) {
    let items: Vec<ListItem> = app
        .get_filtered_items()
        .iter()
        .map(|item| {
            let status_icon = if item.completed { "⬢" } else { "⬡" };
            let important_marker = if item.important { "!" } else { " " };
            let mut text_style = Style::default().fg(colors.primary_text);
            let mut icon_style = Style::default().fg(colors.accent);

            if item.completed {
                text_style = text_style.fg(colors.dim_text).add_modifier(Modifier::DIM);
                icon_style = icon_style.fg(colors.dim_text);
            }

            let short_id = if item.id.len() > 4 {
                &item.id[..4]
            } else {
                &item.id
            }
            .to_uppercase();

            let date_str = match (item.start_date, item.end_date) {
                (Some(s), Some(e)) => format!(
                    " ({: >10} -> {: >10})",
                    s.format("%Y-%m-%d"),
                    e.format("%Y-%m-%d")
                ),
                (Some(s), None) => format!(" (Start: {: >10})", s.format("%Y-%m-%d")),
                (None, Some(e)) => format!(" (End: {: >10})", e.format("%Y-%m-%d")),
                (None, None) => "".to_string(),
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:<4} ", short_id),
                    Style::default().fg(colors.dim_text),
                ),
                Span::styled(format!("{} ", status_icon), icon_style),
                Span::styled(
                    format!("{} ", important_marker),
                    if item.important {
                        Style::default().fg(colors.alert)
                    } else {
                        Style::default().fg(colors.bg)
                    },
                ),
                Span::styled(format!("{}", item.content), text_style),
                Span::styled(
                    date_str,
                    Style::default()
                        .fg(colors.dim_text)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(colors.dim_text))
                .title(" Tasks ")
                .title_style(Style::default().fg(colors.dim_text)),
        )
        .highlight_style(
            Style::default()
                .bg(colors.card_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("→ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}
