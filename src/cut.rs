use anyhow::{anyhow, Context, Result};
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;
use text_io::{read, try_read};

use crate::audio_excerpt::AudioExcerpt;
use crate::config::{CHUNK_SIZE, MAX_OFFSET, MIN_OFFSET, NUM_OFFSETS_TO_TRY, READ_BUFFER};
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::wav::extract_audio;
use crate::{
    args::{CutOpts, OffsetOpts, OffsetPosition},
    config,
};

pub fn cut_session(session: RecordingSession, cut_args: &CutOpts) -> Result<()> {
    // For reasons that I don't quite understand, the exact timings
    // of the (accumulated) song lengths and the timings of the buffer audio file
    // drift apart more and more as the recording grows in length.
    // It might be that the bitrade of the audio isnt exactly what it should be,
    // I don't know. This means that finding one cut offset for all songs at once
    // isn't possible for very long recordings. For this reason I split the recording
    // into chunks of N songs for which I determine the offset at once. The previous
    // chunks offset is then used as a guess for the next chunk (this is relevant in practice
    // because we only read small excerpts of the audio file around the cut position,
    // not the entire audio file for the offset calculation - knowing the previous offset
    // gives a good guess where to look for the audio excerpt that contains the cut.
    let mut offset_guess = 0.0;
    for (i, chunk) in get_chunks(&session).iter().enumerate() {
        println!("Cutting chunk no {} with offset {}", i, offset_guess);
        offset_guess = cut_group(chunk, cut_args, offset_guess)?;
    }
    Ok(())
}

// fn print_timestamps_vs_song_lengths(session: &RecordingSession) -> () {
//     let mut acc_length = 0.0;
//     let initial_timestamp = session.timestamps[0];
//     for (song, timestamp) in session.songs.iter().zip(session.timestamps.iter()) {
//         println!(
//             "{:.2} {:.2} {:.2}",
//             (acc_length - (timestamp - initial_timestamp)),
//             acc_length,
//             timestamp - initial_timestamp
//         );
//         acc_length += song.length;
//     }
// }

pub fn get_chunks(session: &RecordingSession) -> Vec<RecordingSession> {
    let mut sessions = vec![];
    if session.songs.len() == 0 {
        return sessions;
    }
    let (chunk_size, num_chunks) = get_chunk_size_and_num_chunks(session.songs.len(), CHUNK_SIZE);
    // Get the first n-1 chunks which are of size chunk_size
    for i in 0..(num_chunks - 1) {
        let first_song_index = i * chunk_size;
        let last_song_index = (i + 1) * chunk_size - 1;
        sessions.push(get_chunk(session, first_song_index, last_song_index));
    }
    // The last chunk is the remaining songs that may be of lower size
    sessions.push(get_chunk(
        session,
        (num_chunks - 1) * chunk_size,
        session.songs.len() - 1,
    ));
    sessions
    // let new_session = RecordingSession {
    //     dir: session.dir.clone(),
    //     timestamps: vec![timestamp.clone()],
    //     songs: vec![song.clone()],
    // };
    // for (song, timestamp) in session.songs.iter().zip(session.timestamps.iter()) {
    //     if sessions.is_empty() || song.album != sessions.last().unwrap().0.songs[0].album {
    //         let new_session = RecordingSession {
    //             dir: session.dir.clone(),
    //             timestamps: vec![timestamp.clone()],
    //             songs: vec![song.clone()],
    //         };
    //         sessions.push((new_session, song.album.clone()));
    //     } else {
    //         sessions.last_mut().unwrap().0.songs.push(song.clone());
    //     }
    // }
    // sessions
}

fn get_chunk_size_and_num_chunks(num_songs: usize, chunk_size: usize) -> (usize, usize) {
    if num_songs <= chunk_size {
        (num_songs, 1 as usize)
    } else {
        (chunk_size, (num_songs / chunk_size))
    }
}

pub fn get_chunk(
    session: &RecordingSession,
    first_song_index: usize,
    last_song_index: usize,
) -> RecordingSession {
    RecordingSession {
        dir: session.dir.clone(),
        timestamps: session.timestamps[first_song_index..last_song_index].to_vec(),
        songs: session.songs[first_song_index..last_song_index].to_vec(),
    }
}

pub fn cut_group(group: &RecordingSession, cut_args: &CutOpts, offset_guess: f64) -> Result<f64> {
    let cut_timestamps: Vec<f64> = get_cut_timestamps_from_song_lengths(group, offset_guess);
    let (audio_excerpts, valid_songs) = get_audio_excerpts_and_valid_songs(group, &cut_timestamps)?;
    let offset = match &cut_args.offset {
        OffsetOpts::Interactive => determine_cut_offset(audio_excerpts, cut_timestamps),
        OffsetOpts::Auto => determine_cut_offset(audio_excerpts, cut_timestamps),
        OffsetOpts::Manual(off) => off.position,
    };
    println!("Using offset: {:.3}", offset + offset_guess);
    let mut start_time = group.timestamps[0] + offset + offset_guess;
    for song in valid_songs.iter() {
        let end_time = start_time + song.length;
        cut_song(group, song, start_time, end_time)?;
        start_time = end_time;
    }
    match &cut_args.offset {
        OffsetOpts::Auto => {}
        _ => {
            if !user_happy_with_offset(group)? {
                return cut_group(group, &get_manual_cut_options(), offset_guess);
            }
        }
    }
    Ok(offset + offset_guess)
}

fn get_manual_cut_options() -> CutOpts {
    CutOpts {
        offset: OffsetOpts::Manual(OffsetPosition {
            position: get_offset_interactively(),
        }),
    }
}

fn user_happy_with_offset(session: &RecordingSession) -> Result<bool> {
    playback_session(session)?;
    println!("Are the results good? y/N");
    let answer: Result<String, text_io::Error> = try_read!();
    if let Ok(s) = answer {
        Ok(s == "y")
    } else {
        Ok(false)
    }
}

fn playback_session(session: &RecordingSession) -> Result<()> {
    let album_folder = session.songs[0].get_album_folder(&session.get_music_dir());
    Command::new("vlc")
        .arg(album_folder.to_str().unwrap())
        .output()
        .map(|_| ())
        .context("Failed to run playback program")
}

pub fn get_offset_interactively() -> f64 {
    println!("Enter offset (usually between -2 and 1): ");
    read!()
}

pub fn get_excerpt(buffer_file_name: &Path, cut_time: f64) -> Option<AudioExcerpt> {
    let listen_start_time = cut_time + MIN_OFFSET - READ_BUFFER;
    let listen_end_time = cut_time + MAX_OFFSET + READ_BUFFER;
    extract_audio(buffer_file_name, listen_start_time, listen_end_time).ok()
}

pub fn determine_cut_offset(audio_excerpts: Vec<AudioExcerpt>, cut_timestamps: Vec<f64>) -> f64 {
    // We can assume that some of the songs begin or end with silence.
    // If that is the case then the offset of the cuts should be chosen by finding an offset that
    // puts as many of the cuts at positions where the recording is silent. In other words, the offset is given by
    // the local minimum of the convolution of the volume with a sum of dirac deltas at every cut position.
    let mut min: Option<(f64, f64)> = None;
    for i in 0..NUM_OFFSETS_TO_TRY {
        let offset =
            (i as f64) / (NUM_OFFSETS_TO_TRY as f64) * (MAX_OFFSET - MIN_OFFSET) + MIN_OFFSET;
        let total_volume: f64 = cut_timestamps
            .iter()
            .zip(audio_excerpts.iter())
            .map(|(cut_time, audio_excerpt)| audio_excerpt.get_volume_at(cut_time + offset))
            .sum();
        if let Some((min_volume, _)) = min {
            if total_volume < min_volume {
                min = Some((total_volume, offset));
            }
        } else {
            min = Some((total_volume, offset));
        };
    }
    min.unwrap().1
}

pub fn get_audio_excerpts_and_valid_songs<'a>(
    session: &'a RecordingSession,
    cut_timestamps: &[f64],
) -> Result<(Vec<AudioExcerpt>, Vec<&'a Song>)> {
    let mut audio_excerpts = Vec::new();
    let mut valid_songs = Vec::new();

    for (song, cut) in session.songs.iter().zip(cut_timestamps.iter()) {
        let audio_excerpt = get_excerpt(&session.get_buffer_file(), *cut);
        if let Some(excerpt) = audio_excerpt {
            audio_excerpts.push(excerpt);
            valid_songs.push(song);
        } else {
            return Err(anyhow!(
                "Found invalid song: {} {} {}",
                &song.artist,
                &song.album,
                &song.title
            ));
        }
    }
    Ok((audio_excerpts, valid_songs))
}

pub fn get_cut_timestamps_from_song_lengths(
    session: &RecordingSession,
    offset_guess: f64,
) -> Vec<f64> {
    session
        .songs
        .iter()
        .scan(session.timestamps[0] + offset_guess, |acc, song| {
            *acc += song.length;
            Some(*acc)
        })
        .collect()
}

pub fn cut_song(
    session: &RecordingSession,
    song: &Song,
    start_time: f64,
    end_time: f64,
) -> Result<()> {
    let difference = end_time - start_time;
    let source_file = session.get_buffer_file();
    let target_file = song.get_target_file(&session.get_music_dir());
    create_dir_all(target_file.parent().unwrap())
        .context("Failed to create subfolders of target file")?;
    println!(
        "Cutting song: {:.2}+{:.2}: {} to {}",
        start_time,
        difference,
        song,
        target_file.to_str().unwrap()
    );
    Command::new("ffmpeg")
        .arg("-ss")
        .arg(format!("{}", start_time))
        .arg("-t")
        .arg(format!("{}", difference))
        .arg("-i")
        .arg(source_file.to_str().unwrap())
        .arg("-metadata")
        .arg(format!("title={}", &song.title))
        .arg("-metadata")
        .arg(format!("album={}", &song.album))
        .arg("-metadata")
        .arg(format!("artist={}", &song.artist))
        .arg("-metadata")
        .arg(format!("albumartist={}", &song.artist))
        .arg("-metadata")
        .arg(format!("track={}", &song.track_number))
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .output()
        .map(|_| ())
        .context(format!(
            "Failed to cut song: {} {} {} ({}+{})",
            &song.title, &song.album, &song.artist, start_time, difference,
        ))
}
