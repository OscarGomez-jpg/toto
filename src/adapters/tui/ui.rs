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
        CurrentScreen::Main => "a:add  e:edit  c:done  S-j/k:move  g/G:top/bot  ^d/^u:pg  ^l:clear  /:search  q:quit",
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
        
        // 15% height limit, at least 5 lines
        let max_popup_height = (terminal_height as f32 * 0.15).max(5.0) as u16;
        let popup_width = (terminal_width as f32 * 0.6).max(30.0) as u16;
        let inner_width = popup_width.saturating_sub(2);
        
        let text_len = app.input.graphemes(true).count();
        let total_content_len = text_len + 2; // for "> "
        
        let lines_needed = if inner_width > 0 {
            ((total_content_len as u16 + inner_width - 1) / inner_width).max(1) + 2 // +2 for borders
        } else {
            5
        };
        let popup_height = lines_needed.min(max_popup_height);
        let area = centered_rect_fixed(popup_width, popup_height, f.area());
        f.render_widget(Clear, area);
        
        let title = if app.current_screen == CurrentScreen::Adding {
            " Add Task "
        } else {
            " Edit Task "
        };
        
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent_blue))
            .bg(card_bg)
            .title(title)
            .title_style(Style::default().fg(accent_blue));

        // Scroll calculation
        let max_inner_height = popup_height.saturating_sub(2);
        let cursor_pos = total_content_len as u16;
        let cursor_line = if inner_width > 0 { cursor_pos / inner_width } else { 0 };
        let scroll = if cursor_line >= max_inner_height {
            cursor_line - max_inner_height + 1
        } else {
            0
        };

        let input = Paragraph::new(format!("> {}", app.input))
            .style(Style::default().fg(primary_text))
            .block(input_block)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));
        
        f.render_widget(input, area);

        // Cursor positioning
        if inner_width > 0 {
            let cursor_x = cursor_pos % inner_width;
            let display_y = cursor_line.saturating_sub(scroll);
            f.set_cursor_position((
                area.x + 1 + cursor_x,
                area.y + 1 + display_y
            ));
        }
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
