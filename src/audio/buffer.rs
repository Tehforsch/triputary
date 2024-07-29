use std::{fs::File, io::BufReader, path::PathBuf};

use hound::WavReader;

use super::time::AudioTime;

pub struct Buffer {
    reader: WavReader<BufReader<File>>,
}

#[derive(Debug)]
pub struct OutOfBoundsError;

impl From<std::io::Error> for OutOfBoundsError {
    fn from(_error: std::io::Error) -> OutOfBoundsError {
        OutOfBoundsError {}
    }
}

impl Buffer {
    pub fn new(buffer_file: PathBuf) -> Buffer {
        let reader = hound::WavReader::open(buffer_file).unwrap();
        Self { reader }
    }

    pub fn spec(&self) -> hound::WavSpec {
        self.reader.spec()
    }

    fn extract_audio(
        &mut self,
        start: AudioTime,
        end: AudioTime,
    ) -> Result<Vec<i16>, OutOfBoundsError> {
        let num_samples = (end - start).interleaved_sample_num;
        self.reader.seek(start.frame_num())?;
        let samples_interleaved: Vec<i16> = self
            .reader
            .samples::<i16>()
            .take(num_samples as usize)
            .collect::<Result<Vec<_>, hound::Error>>()
            .map_err(|_| OutOfBoundsError)?;
        if samples_interleaved.len() as u32 != num_samples {
            Err(OutOfBoundsError {})
        } else {
            Ok(samples_interleaved)
        }
    }

    pub fn get_volume_at(&mut self, time: AudioTime) -> Result<f64, OutOfBoundsError> {
        pub static NUM_SAMPLES_PER_AVERAGE_VOLUME: usize = 4000;
        let start = time
            - AudioTime::from_sample_and_spec(
                (NUM_SAMPLES_PER_AVERAGE_VOLUME / 2) as u32,
                self.spec(),
            );
        let end = time
            + AudioTime::from_sample_and_spec(
                (NUM_SAMPLES_PER_AVERAGE_VOLUME / 2) as u32,
                self.spec(),
            );
        let inv_len = 1.0 / ((end.interleaved_sample_num - start.interleaved_sample_num) as f64);
        let inv_i16 = 1.0 / (i16::MAX as f64);
        let samples = self.extract_audio(start, end)?;
        let average: f64 = samples
            .iter()
            .map(|x| (*x as f64).abs() * inv_len * inv_i16)
            .sum::<f64>();
        Ok(average)
    }
}
