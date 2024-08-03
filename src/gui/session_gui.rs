use crate::audio::{Cutter, CuttingStrategy, DbusLengthsStrategy, WavFileReader};
use crate::recording_session::{RecordingSession, RecordingSessionWithPath, SessionPath};
use anyhow::Result;
use iced::widget::{row, Canvas, Column};
use iced::Element;

use super::plot::Plot;
use super::Message;

pub struct SessionGui {
    plots: Vec<Plot>,
    reader: WavFileReader,
    session: RecordingSession,
    cutter: Cutter,
}

fn load_plots(reader: &mut WavFileReader, session: &RecordingSession) -> Vec<Plot> {
    // The initial guess for the timestamps
    let timestamps = (DbusLengthsStrategy).get_timestamps(reader, session);
    let mut plots: Vec<_> = (0..session.songs.len())
        .map(|i| {
            let before = if i == 0 {
                None
            } else {
                Some(&session.songs[i])
            };
            let after = session.songs.get(i + 1);
            let timing = timestamps[i];
            Plot::new(reader, before.cloned(), after.cloned(), timing)
        })
        .collect();
    if !session.songs.is_empty() {
        plots.push(Plot::new(
            reader,
            session.songs.last().cloned(),
            None,
            *timestamps.last().unwrap(),
        ))
    }
    plots
}

impl SessionGui {
    pub fn new(path: SessionPath) -> Result<SessionGui> {
        let session = RecordingSessionWithPath::load_from_dir(&path.0)?;
        let mut reader = hound::WavReader::open(session.path.get_buffer_file())?;
        let cutter = Cutter::new(&path.0);
        let plots = load_plots(&mut reader, &session.session);
        Ok(Self {
            reader,
            session: session.session,
            plots,
            cutter,
        })
    }

    pub fn view(&self) -> Element<Message> {
        let canvases: Vec<_> = self
            .plots
            .iter()
            .map(|plot| Canvas::new(plot).width(500).height(500).into())
            .collect();
        let x: Element<Message> = Column::with_children(canvases).into();
        row![x].into()
    }
}
