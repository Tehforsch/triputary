mod cli;
mod config_file;
mod service;

use self::{cli::CliOpts, config_file::ConfigFile};
use crate::recording::SoundServer;
use clap::Parser;
use log::{error, info};
use std::path::PathBuf;

pub use self::service::Service;
pub use cli::Command;

#[derive(Clone)]
pub struct Opts {
    pub output_dir: PathBuf,
    pub service: Service,
    pub sound_server: SoundServer,
    pub command: Command,
    pub verbosity: usize,
}

impl Opts {
    pub fn new(opts: CliOpts, config_file: Option<ConfigFile>) -> Opts {
        let service = opts
            .service
            .or(config_file.as_ref().and_then(|file| file.service))
            .unwrap_or_else(|| {
                let service = Service::default();
                info!(
                    "No service specified in command line options or config file. Using default: {:?}.", service
                );
                service
            });
        let sound_server = opts
            .sound_server
            .or(config_file.as_ref().and_then(|file| file.sound_server))
            .unwrap_or_else(|| {
                let service = SoundServer::default();
                info!(
                    "No sound server specified in command line options or config file. Using default: {:?}.", service
                );
                service
            });
        let output_dir = opts
            .output_dir
            .or(config_file.as_ref().map(|file| file.output_dir.clone()))
            .unwrap_or_else(|| {
panic!("Need an output folder - either pass it as a command line argument or specify it in the config file (probably ~/.config/striputary/config.yaml")
            })
            ;
        Opts {
            output_dir,
            service,
            sound_server,
            command: opts.command,
            verbosity: opts.verbosity,
        }
    }

    pub(crate) fn from_config_and_cli() -> Opts {
        let opts = CliOpts::parse();
        let config_file = ConfigFile::read();
        if let Err(ref e) = config_file {
            error!("{}", e);
        }
        Self::new(opts, config_file.ok())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::config_file::ConfigFile;
    use super::CliOpts;
    use crate::config::{Opts, Service};
    use crate::Command;

    fn test_opts() -> CliOpts {
        CliOpts {
            output_dir: Some("".into()),
            service: None,
            sound_server: None,
            verbosity: 0,
            command: Command::Record,
        }
    }

    fn test_config_file() -> ConfigFile {
        ConfigFile {
            output_dir: "from_config_file".into(),
            service: None,
            sound_server: None,
        }
    }

    #[test]
    fn service_set_properly() {
        use Service::*;
        let mut p_opts = CliOpts {
            service: Some(SpotifyChromium),
            ..test_opts()
        };
        let config_file = ConfigFile {
            service: Some(SpotifyChromium),
            ..test_config_file()
        };

        let opts = Opts::new(p_opts.clone(), None);
        assert_eq!(opts.service, SpotifyChromium);

        p_opts.service = Some(SpotifyNative);
        let opts = Opts::new(p_opts.clone(), None);
        assert_eq!(opts.service, SpotifyNative);

        p_opts.service = None;
        let opts = Opts::new(p_opts.clone(), None);
        assert_eq!(opts.service, Service::default());

        p_opts.service = None;
        let opts = Opts::new(p_opts.clone(), Some(config_file));
        assert_eq!(opts.service, SpotifyChromium);
    }

    #[test]
    fn output_dir_set_properly() {
        let mut p_opts = CliOpts {
            output_dir: Some("from_cli".into()),
            ..test_opts()
        };
        let config_file = ConfigFile {
            output_dir: "from_config_file".into(),
            ..test_config_file()
        };
        let opts = Opts::new(p_opts.clone(), None);
        assert!(opts.output_dir == Path::new("from_cli").to_owned());
        let opts = Opts::new(p_opts.clone(), Some(config_file.clone()));
        assert!(opts.output_dir == Path::new("from_cli").to_owned());
        p_opts.output_dir = None;
        let opts = Opts::new(p_opts.clone(), Some(config_file));
        assert!(opts.output_dir == Path::new("from_config_file").to_owned());
    }

    #[test]
    #[should_panic(expected = "Need an output folder")]
    fn panic_if_output_dir_not_set() {
        let p_opts = CliOpts {
            output_dir: None,
            service: None,
            sound_server: None,
            verbosity: 0,
            command: Command::Record,
        };
        let _opts = Opts::new(p_opts.clone(), None);
    }
}
