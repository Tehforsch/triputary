mod plot;

use crate::config::Config;
use iced::widget::{button, column, row, text, Canvas, Column};
use iced::{Alignment, Element};

use self::plot::Plot;

pub struct Gui {
    plots: Vec<Plot>,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            plots: vec![Plot::new()],
        }
    }
}

impl Gui {
    pub fn run(_: &Config) {
        iced::run("Striputary", Self::update, Self::view).unwrap();
    }
}

impl Gui {
    fn update(&mut self, m: Message) {
        match m {
            Message::Inc => {
                self.plots.push(Plot::new());
            }
            _ => {}
        }
    }

    fn view(&self) -> Element<Message> {
        // Finally, we simply use our `Circle` to create the `Canvas`!
        // The buttons
        let increment = button("+").on_press(Message::Inc);
        // let decrement = button("-").on_press(Message::Dec);

        // The layout
        let canvases: Vec<_> = self
            .plots
            .iter()
            .map(|plot| Canvas::new(plot).width(500).height(500).into())
            .collect();
        let x: Element<Message> = Column::with_children(canvases).into();
        row![x, increment].into()
    }
}

#[derive(Clone, Debug)]
enum Message {
    Inc,
    Dec,
}
