use iced::{
    button, Color};

pub struct ButtonStyle {
    pub is_accessible: bool,
}

impl button::StyleSheet for ButtonStyle {
    fn active(&self) -> button::Style {
        if self.is_accessible {
            button::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.0, 1.0, 0.0))),
                ..Default::default()
            }
        } else {
            button::Style {
                background: Some(iced::Background::Color(Color::from_rgb(1.0, 0.0, 0.0))),
                ..Default::default()
            }
        }
    }
}
