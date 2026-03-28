use crate::adapters::tui::app::{App, CurrentScreen, InputFocus};
use crate::adapters::tui::widgets::colors::Colors;
use crate::adapters::tui::widgets::utils::{centered_rect, centered_rect_fixed};
use ratatui::style::Stylize;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

pub fn draw_popups(f: &mut Frame, app: &mut App, colors: &Colors) {
    match app.current_screen {
        CurrentScreen::Adding | CurrentScreen::Editing => draw_input_popup(f, app, colors),
        CurrentScreen::ConfirmingDelete => draw_delete_popup(f, app, colors),
        _ => {}
    }
}

fn draw_input_popup(f: &mut Frame, app: &mut App, colors: &Colors) {
    let popup_width = (f.area().width as f32 * 0.7).max(40.0) as u16;
    let popup_height = (f.area().height as f32 * 0.4)
        .max(12.0)
        .min(f.area().height as f32 * 0.8) as u16;
    let area = centered_rect_fixed(popup_width, popup_height, f.area());
    f.render_widget(Clear, area);

    let title = if app.current_screen == CurrentScreen::Adding {
        " Add Task "
    } else {
        " Edit Task "
    };
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .bg(colors.card_bg)
            .title(title),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    // Content Field
    let content_block = Block::default()
        .borders(Borders::ALL)
        .title(" Task Description ")
        .border_style(if app.input_focus == InputFocus::Content {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.dim_text)
        });
    let inner_w = chunks[0].width.saturating_sub(2);
    let text_len = app.input.graphemes(true).count() + 2;
    let cursor_line = if inner_w > 0 {
        (text_len as u16) / inner_w
    } else {
        0
    };
    let max_h = chunks[0].height.saturating_sub(2);
    let scroll = if cursor_line >= max_h {
        cursor_line - max_h + 1
    } else {
        0
    };

    f.render_widget(
        Paragraph::new(format!("> {}", app.input))
            .block(content_block)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        chunks[0],
    );
    if app.input_focus == InputFocus::Content && inner_w > 0 {
        f.set_cursor_position((
            chunks[0].x + 1 + (text_len as u16 % inner_w),
            chunks[0].y + 1 + cursor_line.saturating_sub(scroll),
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
        chunks[1],
    );
    if app.input_focus == InputFocus::StartDate {
        f.set_cursor_position((
            chunks[1].x + 1 + app.start_date_input.graphemes(true).count() as u16,
            chunks[1].y + 1,
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
        chunks[2],
    );
    if app.input_focus == InputFocus::EndDate {
        f.set_cursor_position((
            chunks[2].x + 1 + app.end_date_input.graphemes(true).count() as u16,
            chunks[2].y + 1,
        ));
    }

    f.render_widget(
        Paragraph::new("Tab: Next Field | Enter: Save | Esc: Cancel")
            .style(Style::default().fg(colors.dim_text))
            .alignment(Alignment::Center),
        chunks[3],
    );
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
