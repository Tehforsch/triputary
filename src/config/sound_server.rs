use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum SoundServer {
    #[default]
    Pulseaudio,
    Pipewire,
}

impl FromStr for SoundServer {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is quite ugly, but ensures that the config file string representation
        // is the same as in the command line options (which uses the FromStr impl),
        // without adding any additional dependencies
        serde_yaml::from_str(s)
    }
}

impl Display for SoundServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Similar to the from_str implementation, this is ugly but consistent.
        write!(f, "{}", serde_yaml::to_string(self).unwrap())
    }
}
