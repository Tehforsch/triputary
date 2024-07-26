use std::path::Path;

use crate::{config::Config, recording_session::RecordingSessionWithPath};

use super::{
    buffer::Buffer,
    cut::{cut_song, CutInfo},
    cutting_strategy::CuttingStrategy,
};

pub struct Cutter {
    buffer: Buffer,
    session: RecordingSessionWithPath,
}

impl Cutter {
    pub fn new(_: &Config, path: &Path) -> Self {
        let session = RecordingSessionWithPath::load_from_dir(path).unwrap();
        let buffer = Buffer::new(session.path.get_buffer_file());
        Self { buffer, session }
    }

    fn get_cuts(&self, s: impl CuttingStrategy) -> Vec<CutInfo> {
        let timestamps = s.get_timestamps(&self.buffer, &self.session.session);
        assert_eq!(timestamps.len(), self.session.session.songs.len() + 1);
        timestamps
            .iter()
            .zip(timestamps[1..].iter())
            .zip(self.session.session.songs.iter())
            .map(|((start, end), song)| CutInfo::new(&self.session, song, *start, *end))
            .collect()
    }

    pub fn cut(&self, s: impl CuttingStrategy) {
        let cuts = self.get_cuts(s);
        for cut in cuts {
            cut_song(&cut).unwrap();
        }
    }
}
