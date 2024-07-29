use std::fmt::Display;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use log::info;
use serde::Deserialize;
use serde::Serialize;

use super::time::AudioTime;
use crate::consts::{self};
use crate::recording_session::RecordingSessionWithPath;
use crate::song::Song;

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
    cut: Cut,
}

impl CutInfo {
    pub fn new(
        session: &RecordingSessionWithPath,
        song: &Song,
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
                song: song.clone(),
                end_time_secs: end_time.time,
            },
        }
    }
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

fn get_cut_command(info: &CutInfo) -> Result<Command> {
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
    command
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    Ok(command)
}

pub fn cut_multiple_songs(mut cuts: Vec<CutInfo>) -> Result<()> {
    const MAX_NUM_PROCESSES: usize = 10;
    let mut handles = vec![];
    let pop = |cuts: &mut Vec<CutInfo>| {
        if cuts.is_empty() {
            None
        } else {
            Some(cuts.remove(0))
        }
    };
    while !handles.is_empty() || !cuts.is_empty() {
        if handles.len() < MAX_NUM_PROCESSES {
            if let Some(cut) = pop(&mut cuts) {
                let mut command = get_cut_command(&cut)?;
                handles.push(command.spawn().expect("Failed to cut song."));
            }
        }
        let mut finished: Vec<_> = handles
            .iter_mut()
            .map(|handle| match handle.try_wait().unwrap() {
                Some(status) => {
                    if status.success() {
                        true
                    } else {
                        panic!("Cut process failed with {}", status);
                    }
                }
                None => false,
            })
            .collect();
        let (_, still_running): (Vec<_>, Vec<_>) =
            handles.into_iter().partition(|_| finished.remove(0));
        handles = still_running;
    }
    Ok(())
}
