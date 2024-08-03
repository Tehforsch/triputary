use std::path::PathBuf;

use iced::{
    widget::{button, column, row, Column},
    Element,
};
use log::error;

use crate::{
    config::Config,
    recording_session::SessionPath,
    session_manager::{SessionIdentifier, SessionManager},
};

use super::Message;

pub struct SessionSelector {
    sessions: Vec<(SessionIdentifier, PathBuf)>,
}

impl SessionSelector {
    pub fn new(config: &Config) -> SessionSelector {
        let manager = SessionManager::new(&config.output_dir);
        let sessions = manager.iter_relative_paths_with_indices().collect();
        Self { sessions }
    }

    pub fn view(&self) -> Element<Message> {
        error!("Taking only 20 elements since iced crashes otherwise?");
        Column::with_children(self.sessions.iter().take(20).map(|(id, path)| {
            button("+")
                .on_press(Message::SelectSession(path.into()))
                .into()
        }))
        .into()
    }
}
