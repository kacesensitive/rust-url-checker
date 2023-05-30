use iced::{
    button, scrollable, Application, Button, Clipboard, Column, Command, Container,
    Element, Length, Row, Text
};

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use super::button_style::ButtonStyle;

#[derive(Debug, Clone)]
enum URLStatus {
    NotChecked,
    Checking,
    Accessible,
    Inaccessible(String),
}

#[derive(Deserialize)]
struct Urls {
    urls: Vec<String>,
}

fn read_urls_from_json(file_name: &str) -> Result<Vec<String>, String> {
    let path = Path::new(file_name);
    let mut file = File::open(&path)
        .map_err(|err| format!("Could not open file: {:?}", err))?;
    let mut data = String::new();
    file.read_to_string(&mut data)
        .map_err(|err| format!("Could not read file: {:?}", err))?;

    let Urls { urls } = serde_json::from_str(&data)
        .map_err(|err| format!("Could not parse JSON: {:?}", err))?;
    Ok(urls)
}

pub struct URLChecker {
    url_checks: Vec<(String, URLStatus)>,
    buttons: Vec<button::State>,
    check_all_button: button::State,
    reset_button: button::State, 
    error_message: String,
    copy_button: button::State,
    scroll: scrollable::State,
    startup_error: Option<String>,
}


#[derive(Debug, Clone)]
pub enum Message {
    CheckURL(usize),
    CheckedURL(usize, Result<(), String>),
    CheckAllURLs,
    ResetAll,
    CopyError,
}

impl Application for URLChecker {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (URLChecker, Command<Message>) {
        let urls = match read_urls_from_json("src/urls.json") {
            Ok(urls) => urls,
            Err(err) => {
                return (
                    URLChecker {
                        url_checks: Vec::new(),
                        buttons: Vec::new(),
                        check_all_button: button::State::new(),
                        reset_button: button::State::new(),
                        error_message: String::new(),
                        copy_button: button::State::new(),
                        scroll: scrollable::State::new(),
                        startup_error: Some(format!("Could not read URLs from JSON file: {}", err)),
                    },
                    Command::none(),
                )
            }
        };
        
        let url_checks: Vec<(String, URLStatus)> = urls
            .into_iter()
            .map(|url| (url, URLStatus::NotChecked))
            .collect();
    
        let buttons = vec![button::State::new(); url_checks.len()];
    
        (
            URLChecker {
                url_checks,
                buttons,
                check_all_button: button::State::new(),
                reset_button: button::State::new(),
                error_message: String::new(),
                copy_button: button::State::new(),
                scroll: scrollable::State::new(),
                startup_error: None,   // <-- ADD THIS LINE
            },
            Command::none(),
        )
    }
    

    fn title(&self) -> String {
        String::from("URL Checker")
    }

    fn update(&mut self, message: Message, clipboard: &mut Clipboard) -> Command<Message> {
        match message {  
            Message::ResetAll => {
                let urls = match read_urls_from_json("src/urls.json") {
                    Ok(urls) => urls,
                    Err(err) => {
                        eprintln!("Could not read URLs from JSON file: {:?}", err);
                        Vec::new()
                    }
                };
                
                self.url_checks = urls
                    .into_iter()
                    .map(|url| (url, URLStatus::NotChecked))
                    .collect();

                self.buttons = vec![button::State::new(); self.url_checks.len()];
                self.error_message = String::new();
                self.scroll = scrollable::State::new();
                self.startup_error = None;
                Command::none()
            },          
            Message::CheckURL(i) => {
                self.url_checks[i].1 = URLStatus::Checking;
                let url = self.url_checks[i].0.clone();
                Command::perform(check_url(url), move |result| Message::CheckedURL(i, result))
            }
            Message::CheckAllURLs => {
                let commands: Vec<_> = self.url_checks
                    .iter_mut()
                    .enumerate()
                    .map(|(i, (url, status))| {
                        *status = URLStatus::Checking;
                        let url = url.clone();
                        Command::perform(check_url(url), move |result| Message::CheckedURL(i, result))
                    })
                    .collect();
    
                Command::batch(commands)
            }
            Message::CheckedURL(i, result) => {
                match result {
                    Ok(()) => self.url_checks[i].1 = URLStatus::Accessible,
                    Err(e) => {
                        if !self.error_message.is_empty() {
                            self.error_message.push('\n');
                        }
                        self.error_message.push_str(&format!("Error checking {}: {}", self.url_checks[i].0, e));
                        self.url_checks[i].1 = URLStatus::Inaccessible(e.clone());
                    }
                };
                Command::none()
            }
            Message::CopyError => {
                clipboard.write(self.error_message.clone());
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let reset_button = Button::new(&mut self.reset_button, Text::new("Reset"))
            .on_press(Message::ResetAll);
            
        let check_all_button = Button::new(&mut self.check_all_button, Text::new("Check All"))
            .on_press(Message::CheckAllURLs);
    
        let url_list = self.url_checks.iter_mut().enumerate().zip(&mut self.buttons).fold(
            Column::new().spacing(20).align_items(iced::Align::Center), |column, ((i, (url, status)), button)| {
                let status_widget = match status {
                    URLStatus::NotChecked => Button::new(button, Text::new("Check")).on_press(Message::CheckURL(i)),
                    URLStatus::Checking => Button::new(button, Text::new("Checking...")).on_press(Message::CheckURL(i)),
                    URLStatus::Accessible => Button::new(button, Text::new("Accessible")).style(ButtonStyle { is_accessible: true }).on_press(Message::CheckURL(i)),
                    URLStatus::Inaccessible(_) => Button::new(button, Text::new("Inaccessible")).style(ButtonStyle { is_accessible: false }).on_press(Message::CheckURL(i)),
                };
    
                column.push(
                    Row::new()
                        .spacing(20)
                        .push(Text::new(url.as_str()).width(Length::FillPortion(1)))
                        .push(status_widget)
                        .align_items(iced::Align::Center)
                )
            },
        );
    
        let error_text = Text::new(&self.error_message).size(30);
    
        let error_text_scrollable = scrollable::Scrollable::new(&mut self.scroll)
            .width(Length::Fill)
            .height(Length::FillPortion(1))
            .push(error_text);
    

        let content = Column::new()
            .spacing(20)
            .align_items(iced::Align::Center)
            .push(Text::new("URL Checker").size(50))
            .push(Text::new(self.startup_error.as_deref().unwrap_or("")))  // <-- UPDATE THIS LINE
            .push(reset_button)
            .push(check_all_button)
            .push(url_list)
            .push(error_text_scrollable)
            .push(
                Button::new(&mut self.copy_button, Text::new("Copy Errors"))
                    .on_press(Message::CopyError),
            );
    
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .padding(20)
            .center_y()
            .into()
    }
}

async fn check_url(mut url: String) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url.insert_str(0, "http://");
    }

    let response = surf::get(url.clone()).await;
    match response {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
