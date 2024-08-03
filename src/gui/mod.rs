mod plot;
mod session_gui;
mod session_selector;

use crate::config::Config;
use crate::recording_session::SessionPath;
use anyhow::Result;
use iced::application::Application;
use iced::widget::row;
use iced::{executor, Command, Element, Settings, Theme};
use log::error;

use self::session_gui::SessionGui;
use self::session_selector::SessionSelector;

pub struct Gui {
    session: Option<SessionGui>,
    session_selector: SessionSelector,
}

impl Gui {
    fn select_session(&mut self, path: SessionPath) -> Result<()> {
        self.session = Some(SessionGui::new(path)?);
        Ok(())
    }

    pub fn start(config: &Config) {
        let settings = Settings::with_flags(config.clone());
        Gui::run(settings).unwrap()
    }
}

impl Application for Gui {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = Config;

    fn update(&mut self, m: Message) -> Command<Message> {
        match m {
            Message::SelectSession(path) => {
                if let Err(e) = self.select_session(path) {
                    error!("{}", e);
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let selector = self.session_selector.view();
        let session_view = self
            .session
            .as_ref()
            .map(|session| session.view())
            .unwrap_or(row![].into());
        row![selector, session_view].into()
        // let decrement = button("-").on_press(Message::Dec);
    }

    fn new(config: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                session_selector: SessionSelector::new(&config),
                session: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Striputary".to_string()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    SelectSession(SessionPath),
}
