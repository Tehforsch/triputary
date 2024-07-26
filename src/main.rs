mod audio;
mod config;
mod consts;
mod data_stream;
mod errors;
// mod gui;
mod recording;
mod recording_session;
mod song;

use std::path::Path;

use anyhow::Result;
use audio::Cutter;
use audio::DbusStrategy;
use config::Command;
use config::Config;
use log::info;
use log::LevelFilter;
use recording::dbus::DbusConnection;
use recording::recorder::Recorder;
use recording_session::SessionPath;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelPadding;
use simplelog::TermLogger;
use simplelog::TerminalMode;
use time::UtcOffset;

use crate::recording_session::get_new_name;

// use crate::gui::StriputaryGui;

fn record(config: &Config) -> Result<()> {
    info!("Using service: {}", config.service);
    info!("Using sound server: {}", config.sound_server);
    let path = SessionPath(get_new_name(&config.output_dir));
    let recorder = Recorder::new(&config, &path)?;
    let _session = recorder.record_new_session()?;
    Ok(())
}

fn monitor_dbus(config: &Config) {
    let conn = DbusConnection::new(&config.service);
    loop {
        for ev in conn.get_new_events() {
            println!("{:?}", ev);
        }
    }
}

// fn run_gui(config: &Config) {
//     let app = StriputaryGui::new(config);
//     let native_options = eframe::NativeOptions::default();
//     eframe::run_native("striputary", native_options, Box::new(|_| Box::new(app)));
// }

fn cut(config: &Config, session_path: &Path) {
    Cutter::new(config, session_path).cut(DbusStrategy);
}

fn init_logging(verbosity: usize) {
    let level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        v => unimplemented!("Unsupported verbosity level: {}", v),
    };

    let local = chrono::Local::now();
    let offset = local.offset();
    let config = ConfigBuilder::default()
        .set_level_padding(LevelPadding::Right)
        .set_time_offset(UtcOffset::from_whole_seconds(offset.local_minus_utc()).unwrap())
        .build();
    TermLogger::init(level, config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();
}

fn main() -> Result<()> {
    let config = Config::from_config_and_cli();
    init_logging(config.verbosity);
    match config.command {
        Command::Record => record(&config)?,
        Command::Cut(ref args) => cut(&config, &args.path),
        Command::MonitorDbus => monitor_dbus(&config),
    }
    Ok(())
}
