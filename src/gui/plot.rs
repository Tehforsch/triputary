use iced::{
    event::{self, Status},
    mouse::{self, Button},
    widget::canvas::{path::Builder, Event, Frame, Geometry, Path, Program, Stroke},
    Point, Rectangle, Renderer, Theme,
};

use crate::{
    audio::{get_volume_at, interpolate, interpolation_factor, AudioTime, WavFileReader},
    song::Song,
};

use super::Message;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 50.0;
const NUM_PLOT_POINTS: usize = 100;

pub struct Plot {
    data: Vec<Point>,
    _song_before: Option<Song>,
    _song_after: Option<Song>,
    start: AudioTime,
    end: AudioTime,
    cut_time: AudioTime,
}

impl Plot {
    pub fn new(
        reader: &mut WavFileReader,
        song_before: Option<Song>,
        song_after: Option<Song>,
        cut_time: AudioTime,
    ) -> Self {
        let delta = AudioTime::from_time_same_spec(3.0, cut_time);
        let start = cut_time - delta;
        let end = cut_time + delta;
        let data = (0..NUM_PLOT_POINTS)
            .map(|i| {
                let f = i as f32 / NUM_PLOT_POINTS as f32;
                let time = interpolate(start, end, f as f64);
                let vol = get_volume_at(reader, time).unwrap_or(0.0) as f32;
                Point::new(f * WIDTH, HEIGHT / 2.0 + vol * HEIGHT)
            })
            .collect();
        Self {
            data,
            _song_before: song_before,
            _song_after: song_after,
            start,
            end,
            cut_time,
        }
    }

    fn pos_to_time(&self, pos: Point) -> AudioTime {
        interpolate(self.start, self.end, (pos.x / WIDTH) as f64)
    }

    fn time_to_pos(&self, time: AudioTime) -> f32 {
        WIDTH * interpolation_factor(self.start, self.end, time) as f32
    }

    pub fn get_plot_path(&self) -> Path {
        let mut path = Builder::new();

        path.move_to(self.data[0]);
        for point in self.data.iter() {
            path.line_to(*point);
        }
        path.build()
    }

    pub fn get_marker_path(&self) -> Path {
        let mut path = Builder::new();
        let x = self.time_to_pos(self.cut_time);
        path.move_to(Point::new(x, 0.0));
        path.line_to(Point::new(x, HEIGHT));
        path.build()
    }

    pub fn set_cut_position(&mut self, time: AudioTime) {
        self.cut_time = time;
    }
}

pub struct PlotMarkerMoved {
    pub time: AudioTime,
}

impl Program<PlotMarkerMoved> for Plot {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (Status, Option<PlotMarkerMoved>) {
        if let Event::Mouse(mouse::Event::ButtonPressed(ev)) = event {
            if ev == Button::Left {
                if let Some(pos) = cursor.position_in(bounds) {
                    return (
                        Status::Ignored,
                        Some(PlotMarkerMoved {
                            time: self.pos_to_time(pos),
                        }),
                    );
                }
            }
        }
        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let plot = self.get_plot_path();
        frame.stroke(&plot, Stroke::default());
        let marker = self.get_marker_path();
        frame.stroke(&marker, Stroke::default());

        // Finally, we produce the geometry
        vec![frame.into_geometry()]
    }
}
