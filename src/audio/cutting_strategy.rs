use crate::recording_session::RecordingSession;

use super::{buffer::Buffer, time::AudioTime};

pub trait CuttingStrategy {
    fn get_timestamps(&self, buffer: &Buffer, session: &RecordingSession) -> Vec<AudioTime>;
}

pub struct DbusStrategy;

impl CuttingStrategy for DbusStrategy {
    fn get_timestamps(&self, buffer: &Buffer, session: &RecordingSession) -> Vec<AudioTime> {
        let spec = buffer.spec();
        session
            .timestamps
            .iter()
            .map(|t| {
                let time_secs = t.time_since_start_micros as f64 / 1e6;
                AudioTime::from_time_and_spec(time_secs, spec)
            })
            .collect()
    }
}
