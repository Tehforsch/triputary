use std::time::Duration;

pub static CONFIG_FILE_NAME: &str = "config.yaml";

pub static STRIPUTARY_SINK_NAME: &str = "striputary";

pub static DEFAULT_BUFFER_FILE: &str = "buffer.wav";
pub static DEFAULT_SESSION_FILE: &str = "session.yaml";
pub static DEFAULT_MUSIC_DIR: &str = "cut";

pub static TIME_AFTER_SESSION_END: Duration = Duration::from_secs(10);

pub static BITRATE: i64 = 192000;
