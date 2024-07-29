use std::path::Path;

use log::{error, warn};

use crate::{config::Config, recording_session::RecordingSessionWithPath};

use super::{
    buffer::{get_volume_at, WavFileReader},
    cut::{cut_multiple_songs, CutInfo},
    cutting_strategy::CuttingStrategy,
    time::AudioTime,
};

pub struct Cutter {
    reader: WavFileReader,
    session: RecordingSessionWithPath,
}

impl Cutter {
    pub fn new(_: &Config, path: &Path) -> Self {
        let session = RecordingSessionWithPath::load_from_dir(path).unwrap();
        let reader = hound::WavReader::open(session.path.get_buffer_file()).unwrap();
        Self { reader, session }
    }

    fn get_cuts(&mut self, s: impl CuttingStrategy) -> Vec<CutInfo> {
        let timestamps = s.get_timestamps(&mut self.reader, &self.session.session);
        assert_eq!(timestamps.len(), self.session.session.songs.len() + 1);
        timestamps
            .iter()
            .zip(timestamps[1..].iter())
            .zip(self.session.session.songs.iter())
            .map(|((start, end), song)| CutInfo::new(&self.session, song, *start, *end))
            .collect()
    }

    pub fn cut(&mut self, s: impl CuttingStrategy) {
        self.filter_invalid_songs();
        let cuts = self.get_cuts(s);
        cut_multiple_songs(cuts).unwrap();
    }

    fn filter_invalid_songs(&mut self) {
        let last_valid_timestamp = self
            .session
            .session
            .timestamps
            .iter()
            .enumerate()
            .take_while(|(_, timestamp)| {
                let time = AudioTime::from_time_and_spec(timestamp.in_secs(), self.reader.spec());
                get_volume_at(&mut self.reader, time).is_ok()
            })
            .map(|(index, _)| index)
            .last();
        match last_valid_timestamp {
            None => error!("No valid timestamp. Most likely a faulty recording"),
            Some(0) => error!("Only one valid timestamp. Most likely a faulty recording"),
            Some(index) => {
                let drained_songs = self.session.session.songs.drain(index - 1..);
                self.session.session.timestamps.drain(index..);
                for song in drained_songs {
                    warn!("Found invalid song: {song:?}");
                }
            }
        }
    }
}
