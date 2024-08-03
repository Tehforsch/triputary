use std::ops;

use hound::WavSpec;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct AudioTime {
    pub time: f64,
    pub interleaved_sample_num: u32,
    pub channels: u16,
    pub sample_rate: u32,
}

impl AudioTime {
    pub fn from_time_and_spec(time: f64, spec: WavSpec) -> AudioTime {
        AudioTime {
            time,
            channels: spec.channels,
            sample_rate: spec.sample_rate,
            interleaved_sample_num: (time * (spec.channels as u32 * spec.sample_rate) as f64)
                as u32,
        }
    }

    pub fn from_sample_and_spec(interleaved_sample_num: u32, spec: WavSpec) -> AudioTime {
        let time =
            (interleaved_sample_num as f64) / (spec.channels as u32 * spec.sample_rate) as f64;
        AudioTime {
            time,
            channels: spec.channels,
            sample_rate: spec.sample_rate,
            interleaved_sample_num,
        }
    }

    pub fn from_time_same_spec(time: f64, audiotime: AudioTime) -> AudioTime {
        AudioTime {
            time,
            channels: audiotime.channels,
            sample_rate: audiotime.sample_rate,
            interleaved_sample_num: (time
                * (audiotime.channels as u32 * audiotime.sample_rate) as f64)
                as u32,
        }
    }

    pub fn frame_num(&self) -> u32 {
        // TODO(major): does this not overflow?
        (self.time * self.sample_rate as f64) as u32
    }
}

pub fn interpolate(start: AudioTime, end: AudioTime, factor: f64) -> AudioTime {
    start + (end - start) * factor
}

pub fn interpolation_factor(start: AudioTime, end: AudioTime, x: AudioTime) -> f64 {
    (x.time - start.time) / (end.time - start.time)
}

impl ops::Sub<AudioTime> for AudioTime {
    type Output = AudioTime;

    fn sub(self, rhs: AudioTime) -> AudioTime {
        assert_eq!(self.sample_rate, rhs.sample_rate);
        assert_eq!(self.channels, rhs.channels);
        AudioTime::from_time_same_spec(self.time - rhs.time, self)
    }
}

impl ops::Add<AudioTime> for AudioTime {
    type Output = AudioTime;

    fn add(self, rhs: AudioTime) -> AudioTime {
        assert_eq!(self.sample_rate, rhs.sample_rate);
        assert_eq!(self.channels, rhs.channels);
        AudioTime::from_time_same_spec(self.time + rhs.time, self)
    }
}

impl ops::Mul<f64> for AudioTime {
    type Output = AudioTime;

    fn mul(self, fac: f64) -> AudioTime {
        AudioTime::from_time_same_spec(self.time * fac, self)
    }
}

impl PartialOrd for AudioTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
