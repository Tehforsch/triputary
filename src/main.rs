mod audio_excerpt;
mod audio_time;
mod config;
mod consts;
mod cut;
mod data_stream;
mod errors;
mod excerpt_collection;
mod gui;
mod recording;
mod recording_session;
mod service;
mod song;
mod wav;

use anyhow::Result;
use config::Command;
use config::Opts;
use gui::session_manager::get_new_name;
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

use crate::gui::StriputaryGui;

fn record(opts: &Opts) -> Result<()> {
    info!("Using service: {}", opts.service);
    let path = SessionPath(get_new_name(&opts.output_dir));
    let recorder = Recorder::new(&opts, &path)?;
    let _session = recorder.record_new_session()?;
    Ok(())
}

fn monitor_dbus(opts: &Opts) {
    let conn = DbusConnection::new(&opts.service);
    loop {
        for ev in conn.get_new_events() {
            println!("{:?}", ev);
        }
    }
}

fn init_logging(verbosity: usize) {
    let level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
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

fn run_gui(opts: &Opts) {
    let app = StriputaryGui::new(opts);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("striputary", native_options, Box::new(|_| Box::new(app)));
}

fn main() -> Result<()> {
    let opts = Opts::from_config_and_cli();
    init_logging(opts.verbosity);
    match opts.command {
        Command::Record => record(&opts)?,
        Command::Cut => run_gui(&opts),
        Command::MonitorDbus => monitor_dbus(&opts),
    }
    Ok(())
}
