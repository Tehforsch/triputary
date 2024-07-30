use eframe::{
    egui::{CentralPanel, Context},
    App, CreationContext, Frame,
};

use crate::config::Config;

pub struct Gui {}

impl Gui {
    pub fn new(_: &CreationContext, _: &Config) -> Self {
        // TODO(minor): Possibly modify appearance here.
        Self {}
    }

    pub fn run(config: &Config) {
        let config = config.clone();
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Striputary",
            native_options,
            Box::new(move |cc| Ok(Box::new(Self::new(cc, &config)))),
        )
        .unwrap();
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
