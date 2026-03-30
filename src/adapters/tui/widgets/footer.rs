use crate::adapters::tui::app::{App, CurrentScreen};
use crate::adapters::tui::widgets::colors::Colors;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
    Frame,
};

pub fn draw_footer(f: &mut Frame, app: &mut App, area: Rect, colors: &Colors) {
    let help = match app.current_screen {
        CurrentScreen::Main => "a:add  e:edit  c:done  h:help  q:quit",
        CurrentScreen::Gantt => "v:main  j/k:scroll  h:help",
        CurrentScreen::Adding => "enter:save  tab:next  esc:cancel",
        CurrentScreen::Editing => "enter:update  tab:next  esc:cancel",
        CurrentScreen::Searching => "enter:done  esc:reset",
        CurrentScreen::ConfirmingDelete => "Confirm delete? (y/n)",
        CurrentScreen::JiraConfiguring => "enter:save  tab:next  esc:cancel",
        CurrentScreen::Help => "Press any key or h to close",
    };
    f.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(colors.dim_text))
            .alignment(Alignment::Center),
        area,
    );
}
