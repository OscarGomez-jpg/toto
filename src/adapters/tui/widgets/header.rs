use crate::adapters::tui::app::{App, CurrentScreen};
use crate::adapters::tui::widgets::colors::Colors;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

pub fn draw_header(f: &mut Frame, app: &mut App, area: Rect, colors: &Colors) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(area);

    let logo = vec![
        Line::from(vec![Span::styled(
            " ▟████▙",
            Style::default().fg(colors.accent),
        )]),
        Line::from(vec![Span::styled(
            " ▝▘ ▟█▘ ",
            Style::default().fg(colors.accent),
        )]),
        Line::from(vec![
            Span::styled("   ▟█▘  ", Style::default().fg(colors.accent)),
            Span::styled(
                " T O T O",
                Style::default()
                    .fg(colors.primary_text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ▟█▘   ", Style::default().fg(colors.accent)),
            Span::styled(
                format!(" [ v{} ]", env!("CARGO_PKG_VERSION")),
                Style::default().fg(colors.dim_text),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(logo), chunks[0]);

    let search_bar = Paragraph::new(format!("  Search: {}", app.search_query))
        .style(Style::default().fg(colors.primary_text))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(
                    if app.current_screen == CurrentScreen::Searching {
                        colors.accent
                    } else {
                        colors.dim_text
                    },
                )),
        );
    f.render_widget(search_bar, chunks[1]);

    if app.current_screen == CurrentScreen::Searching {
        f.set_cursor_position((
            chunks[1].x + app.search_query.graphemes(true).count() as u16 + 10,
            chunks[1].y,
        ));
    }
}
