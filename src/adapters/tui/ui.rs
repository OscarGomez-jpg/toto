use crate::adapters::tui::app::{App, CurrentScreen};
use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

pub fn ui(f: &mut Frame, app: &mut App) {
    let bg_color = Color::Rgb(20, 20, 25);
    let card_bg = Color::Rgb(30, 30, 35);
    let primary_text = Color::Rgb(224, 224, 224);
    let dim_text = Color::Rgb(120, 120, 130);
    let accent_blue = Color::Rgb(0, 153, 255);
    let alert_red = Color::Rgb(255, 82, 82);

    let main_block = Block::default().style(Style::default().bg(bg_color).fg(primary_text));
    f.render_widget(main_block, f.area());

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // --- HEADER ---
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(outer_layout[0]);

    let ascii_logo = vec![
        Line::from(vec![Span::styled(
            " ▟████▙",
            Style::default().fg(accent_blue),
        )]),
        Line::from(vec![Span::styled(
            " ▝▘ ▟█▘ ",
            Style::default().fg(accent_blue),
        )]),
        Line::from(vec![
            Span::styled("   ▟█▘  ", Style::default().fg(accent_blue)),
            Span::styled(
                " T O T O",
                Style::default()
                    .fg(primary_text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ▟█▘   ", Style::default().fg(accent_blue)),
            Span::styled(" [ v2.2 ]", Style::default().fg(dim_text)),
        ]),
    ];
    f.render_widget(Paragraph::new(ascii_logo), header_chunks[0]);

    let search_bar = Paragraph::new(format!("  Search: {}", app.search_query))
        .style(Style::default().fg(primary_text))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(
                    if app.current_screen == CurrentScreen::Searching {
                        accent_blue
                    } else {
                        dim_text
                    },
                )),
        );
    f.render_widget(search_bar, header_chunks[1]);

    if app.current_screen == CurrentScreen::Searching {
        f.set_cursor_position((
            header_chunks[1].x + app.search_query.graphemes(true).count() as u16 + 10,
            header_chunks[1].y,
        ));
    }

    // --- MAIN CONTENT ---
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(25)])
        .split(outer_layout[1]);

    let items: Vec<ListItem> = app
        .get_filtered_items()
        .iter()
        .map(|item| {
            let status_icon = if item.completed { "⬢" } else { "⬡" };
            let important_marker = if item.important { "!" } else { " " };
            let mut text_style = Style::default().fg(primary_text);
            let mut icon_style = Style::default().fg(accent_blue);
            if item.completed {
                text_style = text_style.fg(dim_text).add_modifier(Modifier::DIM);
                icon_style = icon_style.fg(dim_text);
            }
            let short_id = if item.id.len() > 4 {
                &item.id[..4]
            } else {
                &item.id
            };

            let date_str = match (item.start_date, item.end_date) {
                (Some(s), Some(e)) => format!(" ({: >10} -> {: >10})", s.format("%Y-%m-%d"), e.format("%Y-%m-%d")),
                (Some(s), None) => format!(" (Start: {: >10})", s.format("%Y-%m-%d")),
                (None, Some(e)) => format!(" (End: {: >10})", e.format("%Y-%m-%d")),
                (None, None) => "".to_string(),
            };

            let content = Line::from(vec![
                Span::styled(format!("{:<4} ", short_id), Style::default().fg(dim_text)),
                Span::styled(format!("{} ", status_icon), icon_style),
                Span::styled(
                    format!("{} ", important_marker),
                    if item.important {
                        Style::default().fg(alert_red)
                    } else {
                        Style::default().fg(bg_color)
                    },
                ),
                Span::styled(format!("{}", item.content), text_style),
                Span::styled(date_str, Style::default().fg(dim_text).add_modifier(Modifier::ITALIC)),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(dim_text))
                .title(" Tasks ")
                .title_style(Style::default().fg(dim_text)),
        )
        .highlight_style(Style::default().bg(card_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("→ ");

    f.render_stateful_widget(list, content_layout[0], &mut app.list_state);

    // Sidebar Stats
    let all_tasks = app.task_service.get_all_tasks().unwrap_or_default();
    let total = all_tasks.len();
    let completed = all_tasks.iter().filter(|t| t.completed).count();
    let now = Local::now();

    let stats_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " PROGRESS",
            Style::default().fg(dim_text),
        )]),
        Line::from(vec![
            Span::styled(" Total      ", Style::default().fg(dim_text)),
            Span::styled(total.to_string(), Style::default().fg(primary_text)),
        ]),
        Line::from(vec![
            Span::styled(" Done       ", Style::default().fg(dim_text)),
            Span::styled(completed.to_string(), Style::default().fg(primary_text)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(" STATUS", Style::default().fg(dim_text))]),
        Line::from(vec![
            Span::styled(" Date       ", Style::default().fg(dim_text)),
            Span::styled(
                now.format("%Y-%m-%d").to_string(),
                Style::default().fg(primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Time       ", Style::default().fg(dim_text)),
            Span::styled(
                now.format("%H:%M:%S").to_string(),
                Style::default().fg(accent_blue),
            ),
        ]),
        Line::from(vec![
            Span::styled(" OS         ", Style::default().fg(dim_text)),
            Span::styled(
                std::env::consts::OS.to_uppercase(),
                Style::default().fg(primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" DB         ", Style::default().fg(dim_text)),
            Span::styled("SQLITE", Style::default().fg(primary_text)),
            Span::styled(
                if (app.ticks / 10) % 2 == 0 {
                    " ●"
                } else {
                    " ○"
                },
                Style::default().fg(accent_blue),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " METRICS",
            Style::default().fg(dim_text),
        )]),
        Line::from(vec![
            Span::styled(" Latency    ", Style::default().fg(dim_text)),
            Span::styled(
                format!("{}ms", (app.ticks % 15) + 10),
                Style::default().fg(accent_blue),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Entropy    ", Style::default().fg(dim_text)),
            Span::styled(
                format!("0x{:04X}", app.ticks % 0xFFFF),
                Style::default().fg(primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Kernel     ", Style::default().fg(dim_text)),
            Span::styled("L-042", Style::default().fg(primary_text)),
        ]),
    ];
    let stats = Paragraph::new(stats_text);
    f.render_widget(stats, content_layout[1]);

    // --- FOOTER ---
    let help_message = match app.current_screen {
        CurrentScreen::Main => {
            "a:add  e:edit  c:done  S-j/k:move  g/G:top/bot  ^d/^u:pg  ^l:clear  /:search  q:quit"
        }
        CurrentScreen::Adding => "enter:save  ^c/^u:clear  ^w:word  esc:cancel",
        CurrentScreen::Editing => "enter:update  ^c/^u:clear  ^w:word  esc:cancel",
        CurrentScreen::Searching => "enter:done  esc:reset",
        CurrentScreen::ConfirmingDelete => "Confirm delete? (y/n)",
    };
    let footer = Paragraph::new(help_message)
        .style(Style::default().fg(dim_text))
        .alignment(Alignment::Center);
    f.render_widget(footer, outer_layout[2]);

    // Popups
    if app.current_screen == CurrentScreen::Adding || app.current_screen == CurrentScreen::Editing {
        let terminal_height = f.area().height;
        let terminal_width = f.area().width;

        // Popup takes 70% width and 40% height (or enough for fields)
        let popup_width = (terminal_width as f32 * 0.7).max(40.0) as u16;
        let popup_height = (terminal_height as f32 * 0.4).max(12.0).min(terminal_height as f32 * 0.8) as u16;
        let area = centered_rect_fixed(popup_width, popup_height, f.area());
        f.render_widget(Clear, area);

        let title = if app.current_screen == CurrentScreen::Adding { " Add Task " } else { " Edit Task " };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent_blue))
            .bg(card_bg)
            .title(title);
        f.render_widget(block, area);

        let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Content
                Constraint::Length(3), // Start Date
                Constraint::Length(3), // End Date
                Constraint::Length(1), // Hint
            ])
            .split(inner_area);

        // 1. Content Field
        let content_block = Block::default()
            .borders(Borders::ALL)
            .title(" Task Description ")
            .border_style(if app.input_focus == crate::adapters::tui::app::InputFocus::Content {
                Style::default().fg(accent_blue)
            } else {
                Style::default().fg(dim_text)
            });

        let inner_width = chunks[0].width.saturating_sub(2);
        let text_len = app.input.graphemes(true).count() + 2;
        let cursor_line = if inner_width > 0 { (text_len as u16) / inner_width } else { 0 };
        let max_inner_h = chunks[0].height.saturating_sub(2);
        let scroll = if cursor_line >= max_inner_h { cursor_line - max_inner_h + 1 } else { 0 };

        let content_input = Paragraph::new(format!("> {}", app.input))
            .block(content_block)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));
        f.render_widget(content_input, chunks[0]);

        if app.input_focus == crate::adapters::tui::app::InputFocus::Content && inner_width > 0 {
            f.set_cursor_position((
                chunks[0].x + 1 + (text_len as u16 % inner_width),
                chunks[0].y + 1 + cursor_line.saturating_sub(scroll)
            ));
        }

        // 2. Start Date Field
        let start_block = Block::default()
            .borders(Borders::ALL)
            .title(" Start Date (YYYY-MM-DD) ")
            .border_style(if app.input_focus == crate::adapters::tui::app::InputFocus::StartDate {
                Style::default().fg(accent_blue)
            } else {
                Style::default().fg(dim_text)
            });
        let start_input = Paragraph::new(app.start_date_input.as_str()).block(start_block);
        f.render_widget(start_input, chunks[1]);

        if app.input_focus == crate::adapters::tui::app::InputFocus::StartDate {
            f.set_cursor_position((
                chunks[1].x + 1 + app.start_date_input.graphemes(true).count() as u16,
                chunks[1].y + 1
            ));
        }

        // 3. End Date Field
        let end_block = Block::default()
            .borders(Borders::ALL)
            .title(" End Date (YYYY-MM-DD) ")
            .border_style(if app.input_focus == crate::adapters::tui::app::InputFocus::EndDate {
                Style::default().fg(accent_blue)
            } else {
                Style::default().fg(dim_text)
            });
        let end_input = Paragraph::new(app.end_date_input.as_str()).block(end_block);
        f.render_widget(end_input, chunks[2]);

        if app.input_focus == crate::adapters::tui::app::InputFocus::EndDate {
            f.set_cursor_position((
                chunks[2].x + 1 + app.end_date_input.graphemes(true).count() as u16,
                chunks[2].y + 1
            ));
        }

        let hint = Paragraph::new("Tab: Next Field | Enter: Save | Esc: Cancel")
            .style(Style::default().fg(dim_text))
            .alignment(Alignment::Center);
        f.render_widget(hint, chunks[3]);

    } else if app.current_screen == CurrentScreen::ConfirmingDelete {

        let area = centered_rect(40, 20, f.area());
        f.render_widget(Clear, area);
        let confirm = Paragraph::new("\nConfirm deletion?\n\n(y) Yes / (n) No")
            .style(Style::default().fg(alert_red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(alert_red))
                    .bg(card_bg)
                    .title(" Warning "),
            )
            .alignment(Alignment::Center);
        f.render_widget(confirm, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(r.width.saturating_sub(width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}
