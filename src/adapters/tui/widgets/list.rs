use crate::adapters::tui::app::App;
use crate::adapters::tui::widgets::colors::Colors;
use crate::adapters::tui::widgets::item_card::draw_card_item;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

pub fn draw_task_list(f: &mut Frame, app: &mut App, area: Rect, colors: &Colors) {
    let mut items = app.get_filtered_items();

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(colors.dim_text))
        .title(" Tasks ")
        .title_style(Style::default().fg(colors.dim_text));

    let list_area = block.inner(area);
    f.render_widget(block, area);

    if items.is_empty() {
        return;
    }

    let item_height = 4;
    let visible_count = (list_area.height / item_height) as usize;

    // TODO: Make the ordering scalable
    items.sort_by(|a, b| {
        match (a.start_date(), b.start_date()) {
            (Some(da), Some(db)) => da.cmp(&db),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| match (a.end_date(), b.end_date()) {
            (Some(da), Some(db)) => da.cmp(&db),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        })
        .then_with(|| a.title().cmp(&b.title()))
        .then_with(|| a.id.cmp(&b.id))
    });

    let selected = app.list_state.selected().unwrap_or(0);

    // Simple scrolling logic
    let mut offset = app.list_offset;
    if selected >= offset + visible_count {
        offset = selected.saturating_sub(visible_count).saturating_add(1);
    } else if selected < offset {
        offset = selected;
    }
    app.list_offset = offset;

    for i in 0..visible_count {
        let item_idx = offset + i;
        if item_idx >= items.len() {
            break;
        }

        let item_area = Rect {
            x: list_area.x,
            y: list_area.y + (i as u16 * item_height),
            width: list_area.width,
            height: item_height,
        };

        draw_card_item(f, &items[item_idx], item_area, colors, item_idx == selected);
    }
}
