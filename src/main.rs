use iced::{Application, Settings};
mod url_checker;
mod button_style;

fn main() -> iced::Result {
    url_checker::URLChecker::run(Settings::default())
}
