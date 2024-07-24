use std::fmt::Display;
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use log::info;
use log::warn;
use serde::Deserialize;
use serde::Serialize;

use crate::audio_excerpt::AudioExcerpt;
use crate::audio_time::AudioTime;
use crate::consts::MAX_OFFSET;
use crate::consts::MIN_OFFSET;
use crate::consts::READ_BUFFER;
use crate::consts::{self};
use crate::excerpt_collection::ExcerptCollection;
use crate::excerpt_collection::NamedExcerpt;
use crate::recording_session::RecordingSessionWithPath;
use crate::song::Song;
use crate::wav::extract_audio;

#[derive(Deserialize, Serialize, Debug)]
pub struct Cut {
    pub start_time_secs: f64,
    pub end_time_secs: f64,
    pub song: Song,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CutInfo {
    buffer_file: PathBuf,
    music_dir: PathBuf,
    pub cut: Cut,
}

impl CutInfo {
    pub fn new(
        session: &RecordingSessionWithPath,
        song: Song,
        start_time: AudioTime,
        end_time: AudioTime,
    ) -> Self {
        let buffer_file = session.path.get_buffer_file();
        let music_dir = session.path.get_music_dir();
        CutInfo {
            buffer_file,
            music_dir,
            cut: Cut {
                start_time_secs: start_time.time,
                song,
                end_time_secs: end_time.time,
            },
        }
    }
}

fn get_excerpt(buffer_file_name: &Path, cut_time: f64) -> Option<AudioExcerpt> {
    let listen_start_time = (cut_time + MIN_OFFSET - READ_BUFFER).max(0.0);
    let listen_end_time = cut_time + MAX_OFFSET + READ_BUFFER;
    extract_audio(buffer_file_name, listen_start_time, listen_end_time).ok()
}

pub fn get_excerpt_collection(session: RecordingSessionWithPath) -> ExcerptCollection {
    let (excerpts, songs) = get_all_valid_excerpts_and_songs(&session);
    let offset_guess = 0.0;
    let excerpts: Vec<NamedExcerpt> = excerpts
        .into_iter()
        .enumerate()
        .map(|(num, excerpt)| {
            let song_before = if num == 0 { None } else { songs.get(num) };
            NamedExcerpt {
                excerpt,
                song_before: song_before.cloned(),
                song_after: songs.get(num).cloned(),
                num,
            }
        })
        .collect();
    ExcerptCollection {
        session,
        excerpts,
        offset_guess,
    }
}

fn get_all_valid_excerpts_and_songs(
    session: &RecordingSessionWithPath,
) -> (Vec<AudioExcerpt>, Vec<Song>) {
    let mut audio_excerpts = Vec::new();
    let mut valid_songs = Vec::new();
    let mut cut_time = session.estimated_time_first_song_secs();
    for song in session.session.songs.iter() {
        let audio_excerpt = get_excerpt(&session.path.get_buffer_file(), cut_time);
        if let Some(excerpt) = audio_excerpt {
            audio_excerpts.push(excerpt);
            valid_songs.push(song.clone());
        } else {
            warn!("Could not extract audio for song: {}. Stopping", song);
            break;
        }
        cut_time += song.length;
    }
    let audio_excerpt_after_last_song = get_excerpt(&session.path.get_buffer_file(), cut_time);
    if let Some(audio_excerpt_after_last_song) = audio_excerpt_after_last_song {
        audio_excerpts.push(audio_excerpt_after_last_song);
    }
    (audio_excerpts, valid_songs)
}

fn add_metadata_arg_if_present<T: Display>(
    command: &mut Command,
    get_str: fn(&T) -> String,
    val: Option<&T>,
) {
    if let Some(val) = val {
        command.arg("-metadata").arg(get_str(val));
    }
}

pub fn cut_song(info: &CutInfo) -> Result<()> {
    let difference = info.cut.end_time_secs - info.cut.start_time_secs;
    let target_file = info.cut.song.get_target_file(&info.music_dir);
    create_dir_all(target_file.parent().unwrap())
        .context("Failed to create subfolders of target file")?;
    info!(
        "Cutting song: {:.2}+{:.2}: {} to {}",
        info.cut.start_time_secs,
        difference,
        info.cut.song,
        target_file.to_str().unwrap()
    );
    let mut command = Command::new("ffmpeg");
    command
        .arg("-ss")
        .arg(format!("{}", info.cut.start_time_secs))
        .arg("-t")
        .arg(format!("{}", difference))
        .arg("-i")
        .arg(info.buffer_file.to_str().unwrap())
        .arg("-c:a")
        .arg("libopus")
        .arg("-b:a")
        .arg(format!("{}", consts::BITRATE));
    add_metadata_arg_if_present(
        &mut command,
        |title| format!("title='{}'", title),
        info.cut.song.title.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |album| format!("album='{}'", album),
        info.cut.song.album.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |artist| format!("artist='{}'", artist),
        info.cut.song.artist.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |artist| format!("albumartist='{}'", artist),
        info.cut.song.artist.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |track_number| format!("track={}", track_number),
        info.cut.song.track_number.as_ref(),
    );
    let out = command
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .output();
    out.map(|_| ()).context(format!(
        "Failed to cut song: {:?} {:?} {:?} ({:?}+{:?}) (is ffmpeg installed?)",
        &info.cut.song.title,
        &info.cut.song.album,
        &info.cut.song.artist,
        info.cut.start_time_secs,
        difference,
    ))
}
