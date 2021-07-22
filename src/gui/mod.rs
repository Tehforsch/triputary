mod config;
mod text;

use self::{
    config::{
        LINE_WIDTH, MARKER_HEIGHT, SONG_HEIGHT, SONG_TEXT_X_DISTANCE, SONG_TEXT_Y_OFFSET,
        SONG_X_END, SONG_X_START, SONG_Y_START, Y_DISTANCE_PER_MOUSEWHEEL_TICK, Y_OFFSET_PER_SONG,
    },
    text::get_text_bundle_for_song,
};
use crate::{
    audio_excerpt::AudioExcerpt,
    config::NUM_OFFSETS_TO_TRY,
    cut::{cut_song, get_named_excerpts, NamedExcerpt},
    recording_session::RecordingSession,
    song::Song,
};
use bevy::{app::AppExit, input::mouse::MouseWheel, prelude::*, render::camera::Camera};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    plugin::ShapePlugin,
    prelude::{DrawMode, FillOptions, GeometryBuilder, PathBuilder, ShapeColors, StrokeOptions},
};

struct ExcerptNum(usize);

struct OffsetMarker(f64);

struct TextPosition {
    x: f32,
    y: f32,
}

struct ScrollPosition(i32);

pub fn run(session: RecordingSession) {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(session)
        .insert_resource(ScrollPosition(0))
        // .insert_resource(Msaa { samples: 8 })
        .add_plugin(ShapePlugin)
        .add_startup_system(initialize_camera_system.system())
        .add_startup_system(add_excerpts_system.system())
        .add_system(show_excerpts_system.system())
        .add_system(text_positioning_system.system())
        .add_system(camera_positioning_system.system())
        .add_system(scrolling_input_system.system())
        .add_system(spawn_offset_markers_system.system())
        .add_system(exit_system.system())
        .add_system(cut_system.system())
        .run();
}

fn add_excerpts_system(mut commands: Commands, session: Res<RecordingSession>) {
    let excerpts = get_named_excerpts(&session);
    for (i, excerpt) in excerpts.into_iter().enumerate() {
        commands.spawn().insert(excerpt).insert(ExcerptNum(i));
        commands
            .spawn()
            .insert(ExcerptNum(i))
            .insert(OffsetMarker(0.0));
    }
}

fn show_excerpts_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    excerpts: Query<(Entity, &NamedExcerpt, &ExcerptNum), Without<Draw>>,
) {
    for (entity, excerpt, num) in excerpts.iter() {
        spawn_path_for_excerpt(&mut commands, excerpt, num, entity);
        let get_y_position = |song_num| song_num as f32 * Y_OFFSET_PER_SONG;
        spawn_text_for_excerpt(
            &mut commands,
            &asset_server,
            &excerpt.song,
            TextPosition {
                x: SONG_X_START - SONG_TEXT_X_DISTANCE,
                y: get_y_position(num.0 + 1),
            },
        );
        spawn_text_for_excerpt(
            &mut commands,
            &asset_server,
            &excerpt.song,
            TextPosition {
                x: SONG_X_END + SONG_TEXT_X_DISTANCE,
                y: get_y_position(num.0),
            },
        );
    }
}

fn spawn_text_for_excerpt(
    commands: &mut Commands,
    asset_server: &AssetServer,
    song: &Song,
    text_position: TextPosition,
) {
    commands
        .spawn()
        .insert_bundle(get_text_bundle_for_song(&asset_server, &song))
        .insert(text_position);
}

fn spawn_path_for_excerpt(
    commands: &mut Commands,
    excerpt: &NamedExcerpt,
    num: &ExcerptNum,
    entity: Entity,
) {
    let path = get_path_for_excerpt(excerpt, num);
    commands
        .entity(entity)
        .insert_bundle(get_shape_bundle_for_path(path, LINE_WIDTH, Color::BLACK));
}

fn get_shape_bundle_for_path(path: PathBuilder, line_width: f32, color: Color) -> ShapeBundle {
    let invisible = Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.0,
    };
    GeometryBuilder::build_as(
        &path.build(),
        ShapeColors::outlined(invisible, color),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(line_width),
        },
        Transform::default(),
    )
}

fn get_path_for_excerpt(excerpt: &NamedExcerpt, num: &ExcerptNum) -> PathBuilder {
    let mut path = PathBuilder::new();
    let values = get_volume_data(&excerpt.excerpt);
    let y_offset = (num.0 as f32) * Y_OFFSET_PER_SONG;
    let width = SONG_X_END - SONG_X_START;
    path.move_to(Vec2::new(SONG_X_START, SONG_Y_START + y_offset));
    for (i, y) in values.iter().enumerate() {
        let x = (i as f32) / (values.len() as f32);
        path.line_to(Vec2::new(
            SONG_X_START + x * width,
            SONG_Y_START + y_offset + (*y as f32) * SONG_HEIGHT,
        ));
    }
    path
}

fn get_volume_data(excerpt: &AudioExcerpt) -> Vec<f64> {
    let width = excerpt.end.time - excerpt.start.time;
    let step_size = width / NUM_OFFSETS_TO_TRY as f64;
    let times = (1..NUM_OFFSETS_TO_TRY).map(|x| excerpt.start.time + (x as f64) * step_size);
    times.map(|time| excerpt.get_volume_at(time)).collect()
}

fn spawn_offset_markers_system(
    mut commands: Commands,
    query: Query<(Entity, &OffsetMarker, &ExcerptNum), Without<Draw>>,
) {
    for (entity, _, num) in query.iter() {
        let mut path = PathBuilder::new();
        let middle = (SONG_X_START + SONG_X_END) * 0.5;
        let y_offset = SONG_Y_START + (num.0 as f32) * Y_OFFSET_PER_SONG;
        path.move_to(Vec2::new(middle, y_offset));
        path.line_to(Vec2::new(middle, y_offset + MARKER_HEIGHT));
        commands
            .entity(entity)
            .insert_bundle(get_shape_bundle_for_path(path, 2.0, Color::RED));
    }
}

fn scrolling_input_system(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut pos: ResMut<ScrollPosition>,
) {
    for event in mouse_wheel.iter() {
        if event.y < 0.0 {
            pos.0 -= 1;
        }
        if event.y > 0.0 {
            pos.0 += 1;
        }
    }
}

fn text_positioning_system(mut query: Query<(&mut Transform, &TextPosition), With<Text>>) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y + SONG_TEXT_Y_OFFSET;
    }
}

fn camera_positioning_system(
    mut camera: Query<&mut Transform, With<Camera>>,
    windows: Res<Windows>,
    scroll_position: Res<ScrollPosition>,
) {
    let window = windows.get_primary().unwrap();
    camera.single_mut().unwrap().translation.x = 0.0;
    camera.single_mut().unwrap().translation.y =
        -window.height() / 2.0 + scroll_position.0 as f32 * Y_DISTANCE_PER_MOUSEWHEEL_TICK;
}

fn initialize_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn cut_system(
    keyboard_input: Res<Input<KeyCode>>,
    session: Res<RecordingSession>,
    offsets: Query<(&OffsetMarker, &ExcerptNum)>,
) {
    for key in keyboard_input.get_just_pressed() {
        if let KeyCode::Return = key {
            let mut offsets: Vec<(&OffsetMarker, &ExcerptNum)> = offsets.iter().collect();
            offsets.sort_by_key(|(_, num)| num.0);
            let mut start_time = session.estimated_time_first_song;
            for (marker, num) in offsets.iter() {
                let song = &session.songs[num.0];
                let end_time = start_time + song.length;
                dbg!(song, start_time, end_time);
                cut_song(&session, song, start_time + marker.0, end_time + marker.0).unwrap();
                start_time = start_time + song.length;
            }
        }
    }
}

fn exit_system(keyboard_input: Res<Input<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    for key in keyboard_input.get_just_pressed() {
        match key {
            KeyCode::Escape | KeyCode::Q => {
                app_exit_events.send(AppExit);
            }
            _ => {}
        }
    }
}
