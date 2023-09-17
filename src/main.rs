pub mod args;
pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod config_file;
pub mod cut;
pub mod data_stream;
pub mod errors;
pub mod excerpt_collection;
pub mod gui;
pub mod recording;
pub mod recording_session;
pub mod run_args;
pub mod service_config;
pub mod song;
pub mod wav;

use std::path::Path;

use anyhow::Result;
use args::Opts;
use clap::Parser;
use config_file::ConfigFile;
use log::error;
use log::info;
use log::LevelFilter;
use service_config::Service;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::TermLogger;
use simplelog::TerminalMode;

use crate::gui::StriputaryGui;

fn main() -> Result<(), anyhow::Error> {
    let args = Opts::parse();
    init_logging(&args);
    let config_file = ConfigFile::read();
    if let Err(ref e) = config_file {
        error!("{}", e);
    }
    let config_file = config_file.ok();
    let output_dir = args
        .output_dir
        .or(config_file.as_ref().map(|file| file.output_dir.clone()));
    let service = args
        .service
        .or(config_file.and_then(|file: ConfigFile| file.service))
        .unwrap_or_else(|| {
            let service = Service::default();
            info!("No service specified in command line args or config file. Using default.");
            service
        });
    info!("Using service: {}", service);
    match output_dir {
        Some(dir) => {
            run_gui(&dir, service);
            Ok(())
        }
        None => panic!("Need an output folder - either pass it as a command line argument or specify it in the config file (probably ~/.config/striputary/config.yaml")
    }
}

fn init_logging(args: &Opts) {
    let level = get_log_level(&args);
    TermLogger::init(
        level,
        ConfigBuilder::default().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}

fn get_log_level(args: &Opts) -> LevelFilter {
    match args.verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        v => unimplemented!("Unsupported verbosity level: {}", v),
    }
}

fn run_gui(dir: &Path, service: Service) {
    let app = StriputaryGui::new(dir, service);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("striputary", native_options, Box::new(|_| Box::new(app)));
}
