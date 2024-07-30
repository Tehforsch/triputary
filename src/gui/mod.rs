use crate::config::Config;

pub struct Gui {}

impl Gui {
    pub fn new(_: &Config) -> Self {
        Self {}
    }

    pub fn run(config: &Config) -> Self {
        let gui = Self::new(config);
        todo!()
        // let native_options = eframe::NativeOptions::default();
        // eframe::run_native("striputary", native_options, Box::new(|_| Box::new(gui)));
    }
}
