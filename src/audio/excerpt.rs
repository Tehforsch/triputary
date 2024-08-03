// use hound::WavSpec;
// use rodio::Source;

// use super::{
//     sample_reader::{get_volume_at, OutOfBoundsError, SampleReader},
//     AudioTime,
// };

// #[derive(Clone)]
// pub struct Excerpt {
//     pub samples: Vec<i16>,
//     pub start: AudioTime,
//     pub end: AudioTime,
//     pub spec: WavSpec,
// }

// impl SampleReader for Excerpt {
//     fn extract_audio(
//         &mut self,
//         start: AudioTime,
//         end: AudioTime,
//     ) -> Result<Vec<i16>, OutOfBoundsError> {
//         let interpolate = |time: AudioTime| {
//             let factor = (time.time - self.start.time) / (self.end.time - self.start.time);
//             let dist = self.end.interleaved_sample_num - self.start.interleaved_sample_num;
//             self.start.interleaved_sample_num as usize + ((dist as f64) * factor) as usize
//         };
//         let full_range = self.start.time..self.end.time;
//         if !full_range.contains(&start.time) || !full_range.contains(&end.time) {
//             return Err(OutOfBoundsError);
//         }
//         let start_index = interpolate(start);
//         let end_index = interpolate(end);
//         Ok(self.samples[start_index..end_index]
//             .iter()
//             .cloned()
//             .collect())
//     }

//     fn start(&self) -> AudioTime {
//         self.start
//     }

//     fn end(&self) -> AudioTime {
//         self.start
//     }

//     fn spec(&self) -> WavSpec {
//         self.spec
//     }
// }

// pub const NUM_PLOT_DATA_POINTS: usize = 100;

// impl Excerpt {
//     pub fn get_volume_plot_data(&mut self) -> Vec<f32> {
//         let times = self.get_sample_times();
//         let spec = self.spec().clone();
//         times
//             .into_iter()
//             .map(move |time| {
//                 get_volume_at(self, AudioTime::from_time_and_spec(time as f64, spec)).unwrap_or(0.0)
//                     as f32
//             })
//             .collect()
//     }

//     pub fn get_sample_times(&self) -> Vec<f32> {
//         let width = self.end().time - self.start().time;
//         let step_size = width as f32 / NUM_PLOT_DATA_POINTS as f32;
//         (1..NUM_PLOT_DATA_POINTS)
//             .map(|x| self.start().time as f32 + (x as f32) * step_size as f32)
//             .collect()
//     }

//     pub fn get_absolute_time_by_relative_progress(&self, pos: f64) -> AudioTime {
//         AudioTime::from_time_and_spec(
//             self.start.time + (self.end.time - self.start.time) * pos,
//             self.spec,
//         )
//     }

//     pub fn get_relative_time_by_relative_progress(&self, pos: f64) -> AudioTime {
//         AudioTime::from_time_and_spec((self.end.time - self.start.time) * pos, self.spec)
//     }

//     pub fn get_relative_time(&self, absolute_time: AudioTime) -> AudioTime {
//         absolute_time - self.start
//     }

//     pub fn get_relative_progress_from_time_offset(&self, time_offset: f64) -> f64 {
//         // time_offset is relative to the center
//         0.5 + (time_offset / (self.end.time - self.start.time))
//     }

//     pub fn get_absolute_time_from_time_offset(&self, time_offset: f64) -> AudioTime {
//         self.get_absolute_time_by_relative_progress(
//             self.get_relative_progress_from_time_offset(time_offset),
//         )
//     }
// }
