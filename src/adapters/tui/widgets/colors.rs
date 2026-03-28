use ratatui::style::Color;

pub struct Colors {
    pub bg: Color,
    pub card_bg: Color,
    pub primary_text: Color,
    pub dim_text: Color,
    pub accent: Color,
    pub alert: Color,
}

impl Colors {
    pub fn new() -> Self {
        Self {
            bg: Color::Rgb(20, 20, 25),
            card_bg: Color::Rgb(30, 30, 35),
            primary_text: Color::Rgb(224, 224, 224),
            dim_text: Color::Rgb(120, 120, 130),
            accent: Color::Rgb(0, 153, 255),
            alert: Color::Rgb(255, 82, 82),
        }
    }
}
