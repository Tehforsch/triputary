use std::{fs::File, io::BufReader, path::PathBuf};

use hound::WavReader;

pub struct Buffer {
    reader: WavReader<BufReader<File>>,
}

impl Buffer {
    pub fn new(buffer_file: PathBuf) -> Buffer {
        let reader = hound::WavReader::open(buffer_file).unwrap();
        Self { reader }
    }

    pub fn spec(&self) -> hound::WavSpec {
        self.reader.spec()
    }

    pub fn extract_audio(_start_time: f64, _end_time: f64) {
        todo!()
        // let mut reader = hound::WavReader::open(file_path).unwrap();
        // let spec = reader.spec();
        // let start = AudioTime::from_time_and_spec(start_time, spec);
        // let end = AudioTime::from_time_and_spec(end_time, spec);
        // let num_samples = (end - start).interleaved_sample_num;
        // reader.seek(start.frame_num())?;
        // let samples_interleaved: Vec<i16> = reader
        //     .samples::<i16>()
        //     .take(num_samples as usize)
        //     .collect::<Result<Vec<_>, hound::Error>>()?;
        // if samples_interleaved.len() as u32 != num_samples {
        //     Err(MissingSongError {})
        // } else {
        //     Ok(AudioExcerpt {
        //         spec,
        //         samples: samples_interleaved,
        //         start,
        //         end,
        //     })
        // }
    }
}
