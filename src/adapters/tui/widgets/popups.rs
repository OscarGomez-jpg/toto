use crate::adapters::tui::app::{App, CurrentScreen, InputFocus};
use crate::adapters::tui::widgets::colors::Colors;
use crate::adapters::tui::widgets::utils::{centered_rect, centered_rect_fixed};
use log::{debug, info};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

pub fn draw_popups(f: &mut Frame, app: &mut App, colors: &Colors) {
    match app.current_screen {
        CurrentScreen::Adding | CurrentScreen::Editing => draw_input_popup(f, app, colors),
        CurrentScreen::ConfirmingDelete => draw_delete_popup(f, app, colors),
        CurrentScreen::JiraConfiguring => draw_jira_config_popup(f, app, colors),
        CurrentScreen::Help => draw_help_popup(f, app, colors),
        _ => {}
    }
}

fn draw_help_popup(f: &mut Frame, _app: &mut App, colors: &Colors) {
    let popup_width = 60;
    let popup_height = 22;
    let area = centered_rect_fixed(popup_width, popup_height, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .bg(colors.card_bg)
        .title(" Keyboard Shortcuts ");
    f.render_widget(block, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            " General ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors.accent),
        )]),
        Line::from(" q / Esc      : Back / Quit"),
        Line::from(" h            : Toggle Help"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Main Screen ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors.accent),
        )]),
        Line::from(" a / e        : Add / Edit Task"),
        Line::from(" x / d        : Delete Task"),
        Line::from(" c / Enter    : Toggle Completed"),
        Line::from(" i            : Toggle Important"),
        Line::from(" v            : Toggle Gantt View"),
        Line::from(" j / k / Arrows: Move Selection"),
        Line::from(" S-j / S-k    : Move Task Up/Down"),
        Line::from(" g / G        : Top / Bottom"),
        Line::from(" /            : Search"),
        Line::from(" ^s           : Sync Jira"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Calendar (in Add/Edit) ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors.accent),
        )]),
        Line::from(" Arrows       : Navigate Dates"),
        Line::from(" Space        : Select Date"),
        Line::from(" Tab          : Next Field"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().padding(ratatui::widgets::Padding::uniform(1)))
        .alignment(Alignment::Left);
    f.render_widget(help_paragraph, area);
}

fn draw_jira_config_popup(f: &mut Frame, app: &mut App, colors: &Colors) {
    let popup_width = (f.area().width as f32 * 0.8).max(50.0).min(100.0) as u16;
    let popup_height = 21;
    let area = centered_rect_fixed(popup_width, popup_height, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .bg(colors.card_bg)
        .title(" Jira Configuration ");
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Domain
            Constraint::Length(3), // Email
            Constraint::Length(3), // Token
            Constraint::Length(3), // Projects
            Constraint::Length(3), // Labels
            Constraint::Length(1), // Help
        ])
        .split(area);

    // Domain
    let domain_block = Block::default()
        .borders(Borders::ALL)
        .title(" Jira Domain (e.g. your-name.atlassian.net) ")
        .border_style(if app.input_focus == InputFocus::JiraDomain {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    f.render_widget(
        Paragraph::new(app.jira_domain_input.as_str()).block(domain_block),
        chunks[0],
    );

    // Email
    let email_block = Block::default()
        .borders(Borders::ALL)
        .title(" Email ")
        .border_style(if app.input_focus == InputFocus::JiraEmail {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    f.render_widget(
        Paragraph::new(app.jira_email_input.as_str()).block(email_block),
        chunks[1],
    );

    // Token
    let token_block = Block::default()
        .borders(Borders::ALL)
        .title(" API Token ")
        .border_style(if app.input_focus == InputFocus::JiraToken {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    let masked_token = "*".repeat(app.jira_api_token_input.len());
    f.render_widget(Paragraph::new(masked_token).block(token_block), chunks[2]);

    // Projects
    let projects_block = Block::default()
        .borders(Borders::ALL)
        .title(" Projects (comma separated, e.g. PROJ1, PROJ2) ")
        .border_style(if app.input_focus == InputFocus::JiraProjects {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    f.render_widget(
        Paragraph::new(app.jira_projects_input.as_str()).block(projects_block),
        chunks[3],
    );

    // Labels
    let labels_block = Block::default()
        .borders(Borders::ALL)
        .title(" Labels (comma separated, e.g. urgent, v1) ")
        .border_style(if app.input_focus == InputFocus::JiraLabels {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    f.render_widget(
        Paragraph::new(app.jira_labels_input.as_str()).block(labels_block),
        chunks[4],
    );

    f.render_widget(
        Paragraph::new("Tab: Next | Enter: Save & Connect | Esc: Cancel")
            .style(Style::default().fg(colors.dim_text))
            .alignment(Alignment::Center),
        chunks[5],
    );

    // Set cursor
    let focus_chunk = match app.input_focus {
        InputFocus::JiraDomain => Some((chunks[0], &app.jira_domain_input)),
        InputFocus::JiraEmail => Some((chunks[1], &app.jira_email_input)),
        InputFocus::JiraToken => Some((chunks[2], &app.jira_api_token_input)),
        InputFocus::JiraProjects => Some((chunks[3], &app.jira_projects_input)),
        InputFocus::JiraLabels => Some((chunks[4], &app.jira_labels_input)),
        _ => None,
    };

    if let Some((chunk, input)) = focus_chunk {
        f.set_cursor_position((
            chunk.x + 1 + input.graphemes(true).count() as u16,
            chunk.y + 1,
        ));
    }
}

fn draw_input_popup(f: &mut Frame, app: &mut App, colors: &Colors) {
    let popup_width = (f.area().width as f32 * 0.9).max(60.0).min(100.0) as u16;
    let popup_height = (f.area().height as f32 * 0.5)
        .max(15.0)
        .min(f.area().height as f32 * 0.9) as u16;
    let area = centered_rect_fixed(popup_width, popup_height, f.area());
    f.render_widget(Clear, area);

    let title = if app.current_screen == CurrentScreen::Adding {
        " Add Task "
    } else {
        " Edit Task "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .bg(colors.card_bg)
        .title(title);
    f.render_widget(block, area);

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(outer_layout[0]);

    // Content Field
    let content_block = Block::default()
        .borders(Borders::ALL)
        .title(" Task Description ")
        .border_style(if app.input_focus == InputFocus::Content {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });

    let inner_w = left_chunks[0].width.saturating_sub(2);
    let mut cursor_y = 0;
    let mut cursor_x = 0;

    if inner_w > 0 {
        let full_text = format!("> {}", app.input);
        let mut current_line_width = 0;

        for word in full_text.split_inclusive(' ') {
            let word_width = word.graphemes(true).count();
            if current_line_width + word_width <= inner_w as usize {
                current_line_width += word_width;
            } else {
                if word_width > inner_w as usize {
                    let mut remaining = word_width;
                    while remaining > 0 {
                        let space = inner_w as usize - current_line_width;
                        if space == 0 {
                            cursor_y += 1;
                            current_line_width = 0;
                            continue;
                        }
                        let take = space.min(remaining);
                        current_line_width += take;
                        remaining -= take;
                        if remaining > 0 {
                            cursor_y += 1;
                            current_line_width = 0;
                        }
                    }
                } else {
                    cursor_y += 1;
                    current_line_width = word_width;
                }
            }
        }
        cursor_x = current_line_width as u16;
        if cursor_x == inner_w {
            cursor_y += 1;
            cursor_x = 0;
        }
    }

    let max_h = left_chunks[0].height.saturating_sub(2);
    let scroll = if cursor_y >= max_h {
        cursor_y - max_h + 1
    } else {
        0
    };

    f.render_widget(
        Paragraph::new(format!("> {}", app.input))
            .block(content_block)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        left_chunks[0],
    );

    if app.input_focus == InputFocus::Content && inner_w > 0 {
        f.set_cursor_position((
            left_chunks[0].x + 1 + cursor_x,
            left_chunks[0].y + 1 + cursor_y.saturating_sub(scroll),
        ));
    }

    // Start Date
    let start_block = Block::default()
        .borders(Borders::ALL)
        .title(" Start Date (YYYY-MM-DD) ")
        .border_style(if app.input_focus == InputFocus::StartDate {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    f.render_widget(
        Paragraph::new(app.start_date_input.as_str()).block(start_block),
        left_chunks[1],
    );
    if app.input_focus == InputFocus::StartDate {
        f.set_cursor_position((
            left_chunks[1].x + 1 + app.start_date_input.graphemes(true).count() as u16,
            left_chunks[1].y + 1,
        ));
    }

    // End Date
    let end_block = Block::default()
        .borders(Borders::ALL)
        .title(" End Date (YYYY-MM-DD) ")
        .border_style(if app.input_focus == InputFocus::EndDate {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    f.render_widget(
        Paragraph::new(app.end_date_input.as_str()).block(end_block),
        left_chunks[2],
    );
    if app.input_focus == InputFocus::EndDate {
        f.set_cursor_position((
            left_chunks[2].x + 1 + app.end_date_input.graphemes(true).count() as u16,
            left_chunks[2].y + 1,
        ));
    }

    f.render_widget(
        Paragraph::new("Tab: Next Field | Enter: Save | Esc: Cancel")
            .style(Style::default().fg(colors.dim_text))
            .alignment(Alignment::Center),
        left_chunks[3],
    );

    // Calendar
    if app.input_focus == InputFocus::StartDate || app.input_focus == InputFocus::EndDate {
        let calendar_date = app.get_time_date();
        let mut events = CalendarEventStore::default();
        events.add(
            calendar_date,
            Style::default().bg(colors.accent).fg(colors.bg),
        );

        let calendar = Monthly::new(calendar_date, events)
            .block(Block::default().borders(Borders::ALL).title(" Calendar "))
            .show_surrounding(Style::default().fg(colors.dim_text))
            .show_month_header(Style::default().fg(colors.accent))
            .show_weekdays_header(Style::default().fg(colors.dim_text));

        f.render_widget(calendar, outer_layout[1]);

        let help_text = "Arrows: Navigate | Space: Select Date";
        let help_area = Rect {
            x: outer_layout[1].x,
            y: outer_layout[1].y + outer_layout[1].height.saturating_sub(1),
            width: outer_layout[1].width,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(help_text)
                .style(Style::default().fg(colors.dim_text))
                .alignment(Alignment::Center),
            help_area,
        );
    }
}

fn draw_delete_popup(f: &mut Frame, _app: &mut App, colors: &Colors) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    let confirm = Paragraph::new("\nConfirm deletion?\n\n(y) Yes / (n) No")
        .style(Style::default().fg(colors.alert))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.alert))
                .bg(colors.card_bg)
                .title(" Warning "),
        )
        .alignment(Alignment::Center);
    f.render_widget(confirm, area);
}
