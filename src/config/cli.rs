use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::{Service, SoundServer};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
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
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: usize,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Begin a new recording.
    Record,
    /// Cut a previous recording into individual songs.
    Cut(CutArgs),
    /// Monitor the incoming d-bus messages. Used for
    /// debugging purposes.
    MonitorDbus,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct CutArgs {
    /// Path of the recording session to cut.
    pub path: PathBuf,
}
