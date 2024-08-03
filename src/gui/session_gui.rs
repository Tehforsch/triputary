use crate::audio::{CuttingStrategy, DbusLengthsStrategy, Manual, WavFileReader};
use crate::recording_session::{RecordingSession, RecordingSessionWithPath, SessionPath};
use anyhow::Result;
use iced::widget::{row, Canvas, Column};
use iced::Element;

use super::plot::{Plot, PlotMarkerMoved};
use super::{Message, SetCutPosition};

const CANVAS_WIDTH: f32 = 800.0;
const CANVAS_HEIGHT: f32 = 80.0;

pub struct SessionGui {
    plots: Vec<Plot>,
    reader: WavFileReader,
    session: RecordingSession,
    cuts: Manual,
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
        let plots = load_plots(&mut reader, &session.session);
        let cuts = Manual::new(&mut reader, &session.session, DbusLengthsStrategy);
        Ok(Self {
            reader,
            session: session.session,
            plots,
            cuts,
        })
    }

    pub fn view(&self) -> Element<Message> {
        let canvases: Vec<_> = self
            .plots
            .iter()
            .enumerate()
            .map(|(i, plot)| {
                let c: Element<PlotMarkerMoved> = Canvas::new(plot)
                    .width(CANVAS_WIDTH)
                    .height(CANVAS_HEIGHT)
                    .into();
                c.map(move |message| {
                    Message::SetCutPosition(SetCutPosition {
                        cut_index: i,
                        time: message.time,
                    })
                })
            })
            .collect();
        let x: Element<Message> = Column::with_children(canvases).into();
        row![x].into()
    }

    pub fn set_cut_position(&mut self, pos: SetCutPosition) {
        self.cuts.0[pos.cut_index] = pos.time;
        self.plots[pos.cut_index].set_cut_position(pos.time);
    }
}
