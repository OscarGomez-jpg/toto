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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::tui::app::App;
    use crate::ports::inbound::MockTaskServicePort;
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::Arc;

    #[test]
    fn test_ui_main_screen_snapshot() {
        let mut mock_service = MockTaskServicePort::new();
        mock_service.expect_get_all_tasks().returning(|| Ok(vec![]));

        let mut app = App::new(Arc::new(mock_service));
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| ui(f, &mut app)).unwrap();

        let view = format!("{:?}", terminal.backend());
        // Scrub dynamic parts
        let scrubbed = scrub_ui_view(&view);
        insta::assert_snapshot!(scrubbed);
    }

    fn scrub_ui_view(view: &str) -> String {
        let mut result = view.to_string();
        // Scrub Date (YYYY-MM-DD and partials like MM-DD)
        let re_date = regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
        result = re_date.replace_all(&result, "YYYY-MM-DD").to_string();
        let re_date_partial = regex::Regex::new(r"\d{2}-\d{2}").unwrap();
        result = re_date_partial.replace_all(&result, "MM-DD").to_string();

        // Scrub Time (HH:MM:SS and partials like :SS)
        let re_time = regex::Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
        result = re_time.replace_all(&result, "HH:MM:SS").to_string();
        let re_time_partial = regex::Regex::new(r":\d{2}").unwrap();
        result = re_time_partial.replace_all(&result, ":XX").to_string();

        // Scrub Latency (XXms)
        let re_latency = regex::Regex::new(r"\d{2}ms").unwrap();
        result = re_latency.replace_all(&result, "XXms").to_string();
        // Scrub Entropy (0xXXXX)
        let re_entropy = regex::Regex::new(r"0x[0-9A-F]{4}").unwrap();
        result = re_entropy.replace_all(&result, "0xXXXX").to_string();
        result
    }

    #[test]
    fn test_ui_gantt_screen_snapshot() {
        let mut mock_service = MockTaskServicePort::new();
        mock_service.expect_get_all_tasks().returning(|| Ok(vec![]));

        let mut app = App::new(Arc::new(mock_service));
        app.current_screen = CurrentScreen::Gantt;
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| ui(f, &mut app)).unwrap();

        let view = format!("{:?}", terminal.backend());
        insta::assert_snapshot!(scrub_ui_view(&view));
    }

    #[test]
    fn test_ui_help_screen_snapshot() {
        let mut mock_service = MockTaskServicePort::new();
        mock_service.expect_get_all_tasks().returning(|| Ok(vec![]));

        let mut app = App::new(Arc::new(mock_service));
        app.current_screen = CurrentScreen::Help;
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| ui(f, &mut app)).unwrap();

        let view = format!("{:?}", terminal.backend());
        insta::assert_snapshot!(scrub_ui_view(&view));
    }

    #[test]
    fn test_ui_jira_config_screen_snapshot() {
        let mut mock_service = MockTaskServicePort::new();
        mock_service.expect_get_all_tasks().returning(|| Ok(vec![]));

        let mut app = App::new(Arc::new(mock_service));
        app.current_screen = CurrentScreen::JiraConfiguring;
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| ui(f, &mut app)).unwrap();

        let view = format!("{:?}", terminal.backend());
        insta::assert_snapshot!(scrub_ui_view(&view));
    }
}
