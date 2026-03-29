use crate::adapters::tui::app::{App, CurrentScreen};
use crate::adapters::tui::widgets::colors::Colors;
use crate::adapters::tui::widgets::footer::draw_footer;
use crate::adapters::tui::widgets::gantt::draw_gantt_chart;
use crate::adapters::tui::widgets::header::draw_header;
use crate::adapters::tui::widgets::list::draw_task_list;
use crate::adapters::tui::widgets::popups::draw_popups;
use crate::adapters::tui::widgets::sidebar::draw_sidebar;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let colors = Colors::new();

    // Base background
    let main_block = Block::default().style(Style::default().bg(colors.bg).fg(colors.primary_text));
    f.render_widget(main_block, f.area());

    // Main Layout (Header, Content, Footer)
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Header
    draw_header(f, app, outer_layout[0], &colors);

    // Content (Task List + Sidebar OR Gantt Chart)
    if app.current_screen == CurrentScreen::Gantt {
        draw_gantt_chart(f, app, outer_layout[1], &colors);
    } else {
        let main_content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(25)])
            .split(outer_layout[1]);

        draw_task_list(f, app, main_content_layout[0], &colors);
        draw_sidebar(f, app, main_content_layout[1], &colors);
    }

    // Footer
    draw_footer(f, app, outer_layout[2], &colors);

    // Overlays
    draw_popups(f, app, &colors);
}
