use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

use hound::WavSpec;
use rodio::OutputStream;
use rodio::Sink;
use rodio::Source;

use crate::audio::AudioTime;

use super::sample_reader::WavFileReader;
use super::SampleReader;

pub struct WavSource {
    spec: WavSpec,
    samples: Vec<i16>,
    position: usize,
}

impl WavSource {
    pub fn new(reader: &mut WavFileReader, start_time: AudioTime, end_time: AudioTime) -> Self {
        let spec = reader.spec();
        let samples = reader.extract_audio(start_time, end_time).unwrap();
        Self {
            spec,
            samples,
            position: 0,
        }
    }
}

impl Source for WavSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.spec.channels
    }

    fn sample_rate(&self) -> u32 {
        self.spec.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for WavSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.samples.get(self.position);
        self.position += 1;
        item.copied()
    }
}

pub fn play_audio(
    buffer: &mut WavFileReader,
    start_time: AudioTime,
    end_time: AudioTime,
) -> PlaybackThreadHandle {
    let source = WavSource::new(buffer, start_time, end_time);
    let stop = Arc::new(AtomicBool::new(false));
    thread::spawn({
        let stop = stop.clone();
        move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(source);
            sink.play();
            while !stop.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(1));
            }
        }
    });
    PlaybackThreadHandle {
        stop,
        start_system_time: SystemTime::now(),
        start_audio_time: start_time,
    }
}

pub struct PlaybackThreadHandle {
    stop: Arc<AtomicBool>,
    start_system_time: SystemTime,
    start_audio_time: AudioTime,
}

impl PlaybackThreadHandle {
    pub fn shut_down(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn get_current_audio_time(&self) -> AudioTime {
        let time_expired = SystemTime::now().duration_since(self.start_system_time);
        let time_expired_secs = time_expired
            .unwrap_or(Duration::from_millis(0))
            .as_secs_f64();
        AudioTime::from_time_same_spec(
            self.start_audio_time.time + time_expired_secs,
            self.start_audio_time,
        )
    }
}
