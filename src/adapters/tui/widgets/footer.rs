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
        CurrentScreen::Main => {
            "a:add  e:edit  c:done  S-j/k:move  g/G:top/bot  ^d/^u:pg  ^l:clear  /:search  q:quit"
        }
        CurrentScreen::Adding => "enter:save  ^c/^u:clear  ^w:word  esc:cancel",
        CurrentScreen::Editing => "enter:update  ^c/^u:clear  ^w:word  esc:cancel",
        CurrentScreen::Searching => "enter:done  esc:reset",
        CurrentScreen::ConfirmingDelete => "Confirm delete? (y/n)",
    };
    f.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(colors.dim_text))
            .alignment(Alignment::Center),
        area,
    );
}
