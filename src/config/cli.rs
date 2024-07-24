use std::path::PathBuf;

use clap::Parser;

use super::{Service, SoundServer};

#[derive(Parser, Debug, Clone)]
pub enum Command {
    /// Begin a new recording.
    Record,
    /// Cut a previous recording into individual songs.
    Cut,
    /// Monitor the incoming d-bus messages. Used for
    /// debugging purposes.
    MonitorDbus,
}

#[derive(clap::StructOpt, Clone)]
#[clap(version)]
pub struct CliOpts {
    /// The output directory to record to.
    /// Passing this argument will override the setting
    /// in the config file.
    pub output_dir: Option<PathBuf>,
    /// The service to record.  Passing this argument will override
    /// the setting in the config file.
    pub service: Option<Service>,
    /// The sound server to use.  Passing this argument will override
    /// the setting in the config file.
    pub sound_server: Option<SoundServer>,
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,
    #[clap(subcommand)]
    pub command: Command,
}
