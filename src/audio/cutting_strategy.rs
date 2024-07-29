use log::debug;

use crate::{
    audio::buffer::{get_volume_at, OutOfBoundsError},
    recording_session::RecordingSession,
    song::Song,
};

use super::{buffer::WavFileReader, time::AudioTime};

pub trait CuttingStrategy {
    fn get_timestamps(
        &self,
        buffer: &mut WavFileReader,
        session: &RecordingSession,
    ) -> Vec<AudioTime>;
}

pub struct DbusStrategy;

impl CuttingStrategy for DbusStrategy {
    fn get_timestamps(
        &self,
        buffer: &mut WavFileReader,
        session: &RecordingSession,
    ) -> Vec<AudioTime> {
        let spec = buffer.spec();
        session
            .timestamps
            .iter()
            .map(|t| AudioTime::from_time_and_spec(t.in_secs(), spec))
            .collect()
    }
}

fn get_cut_timestamps_from_song_lengths(
    songs: &[Song],
    estimated_time_first_song: f64,
) -> impl Iterator<Item = f64> + '_ {
    std::iter::once(estimated_time_first_song).chain(songs.iter().scan(
        estimated_time_first_song,
        |acc, song| {
            *acc += song.length;
            let result = Some(*acc);
            result
        },
    ))
}

pub struct DbusLengthsStrategy;

impl CuttingStrategy for DbusLengthsStrategy {
    fn get_timestamps(
        &self,
        buffer: &mut WavFileReader,
        session: &RecordingSession,
    ) -> Vec<AudioTime> {
        let spec = buffer.spec();
        let first_timestamp = &session.timestamps[0];
        get_cut_timestamps_from_song_lengths(&session.songs, first_timestamp.in_secs())
            .map(|time| AudioTime::from_time_and_spec(time, spec))
            .collect()
    }
}

pub struct SilenceOptimizer;

impl CuttingStrategy for SilenceOptimizer {
    fn get_timestamps(
        &self,
        buffer: &mut WavFileReader,
        session: &RecordingSession,
    ) -> Vec<AudioTime> {
        let guesses = DbusLengthsStrategy::get_timestamps(&DbusLengthsStrategy, buffer, session);
        let offset = optimize_cut_offset(buffer, &guesses);
        guesses.into_iter().map(|time| time + offset).collect()
    }
}

fn optimize_cut_offset(buffer: &mut WavFileReader, guesses: &[AudioTime]) -> AudioTime {
    // We can assume that some of the songs begin or end with silence.
    // If that is the case then the offset of the cuts should be chosen by finding an offset that
    // puts as many of the cuts at positions where the recording is silent. In other words, the offset is given by
    // the local minimum of the convolution of the volume with a sum of dirac deltas at every cut position.
    let spec = buffer.spec();
    let mut min: Option<(f64, AudioTime)> = None;
    let offsets: Vec<_> = (0..NUM_OFFSETS_TO_TRY)
        .map(|i| (i as f64) / (NUM_OFFSETS_TO_TRY as f64) * (MAX_OFFSET - MIN_OFFSET) + MIN_OFFSET)
        .map(|offset| AudioTime::from_time_and_spec(offset, spec))
        .collect();
    for offset in offsets.into_iter() {
        let total_volume: Result<f64, OutOfBoundsError> = guesses
            .iter()
            .map(|time| get_volume_at(buffer, *time + offset))
            .sum();
        if let Ok(total_volume) = total_volume {
            if let Some((min_volume, _)) = min {
                if total_volume < min_volume {
                    min = Some((total_volume, offset));
                }
            } else {
                min = Some((total_volume, offset));
            }
        }
    }
    let cut_quality_estimate = min.unwrap().0 / (guesses.len() as f64);
    debug!("Av. volume at cuts: {:.5}", cut_quality_estimate);
    min.unwrap().1
}

pub static MIN_OFFSET: f64 = -3.;
pub static MAX_OFFSET: f64 = 3.;
pub static NUM_OFFSETS_TO_TRY: i64 = 10000;
