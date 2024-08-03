use iced::{
    mouse,
    widget::canvas::{self, path::Builder, Canvas, Frame, Geometry, Path, Program, Stroke},
    Color, Point, Rectangle, Renderer, Theme,
};

use super::Message;

pub struct Plot {
    data: Vec<Point>,
}

impl Plot {
    pub fn new() -> Self {
        let data = (0..1000)
            .map(|x| Point::new(x as f32, 500.0 * (x as f32 / 100.0).sin()))
            .collect();
        Self { data }
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
