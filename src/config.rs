pub static SINK_NAME: &'static str = "spotifyrec";
pub static SINK_SOURCE_NAME: &'static str = "Spotify";
pub static DEFAULT_BUFFER_FILE: &'static str = "buffer.ogg";
pub static DEFAULT_SESSION_FILE: &'static str = "session.yaml";

pub static BITRATE: i64 = 320;
pub static MAX_OFFSET: f64 = 10.;
pub static MAX_SEEK_ERROR: f64 = 5.0;
pub static NUM_SAMPLES_PER_AVERAGE: usize = 10;