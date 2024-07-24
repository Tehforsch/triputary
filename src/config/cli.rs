use std::path::PathBuf;

use clap::Parser;

use crate::{recording::SoundServer, service::Service};

#[derive(Parser, Debug, Clone)]
pub enum Command {
    Record,
    Cut,
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
