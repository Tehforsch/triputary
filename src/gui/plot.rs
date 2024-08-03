use iced::{
    mouse,
    widget::canvas::{path::Builder, Frame, Geometry, Path, Program, Stroke},
    Point, Rectangle, Renderer, Theme,
};

use crate::{
    audio::{AudioTime, SampleReader, WavFileReader},
    song::Song,
};

use super::Message;

pub struct Plot {
    data: Vec<Point>,
    _song_before: Option<Song>,
    _song_after: Option<Song>,
}

impl Plot {
    pub fn new(
        reader: &mut WavFileReader,
        song_before: Option<Song>,
        song_after: Option<Song>,
        timing: AudioTime,
    ) -> Self {
        let delta = AudioTime::from_time_same_spec(3.0, timing);
        let data = reader
            .extract_audio(timing - delta, timing + delta)
            .unwrap();
        let data = data
            .into_iter()
            .enumerate()
            .map(|(i, vol)| {
                let vol = vol as f32 / i16::MAX as f32;
                Point::new(i as f32, vol * 100.0)
            })
            .collect();
        Self {
            data,
            _song_before: song_before,
            _song_after: song_after,
        }
    }

    pub fn get_path(&self) -> Path {
        let mut path = Builder::new();

        path.move_to(self.data[0]);
        for point in self.data.iter() {
            path.line_to(*point);
        }
        path.build()
    }
}

impl Program<Message> for Plot {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let path = self.get_path();
        frame.stroke(&path, Stroke::default());

        // Finally, we produce the geometry
        vec![frame.into_geometry()]
    }
}
