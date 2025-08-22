use std::f32::consts::TAU;
use bevy::prelude::{Commands, Component, Entity, EventReader, NextState, OnEnter, Ref, Res, ResMut, Single, StateScoped, Visibility, With};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetApp, AssetServer, Assets};
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{on_event, IntoScheduleConfigs};

use crate::camera::{CameraMode, CameraState};
use crate::events::GameEvent;
use crate::level::datafile::DataFile;
use crate::level::loading::new_level;
use crate::screens::Screen;
use crate::terrain::Terrain;

pub mod datafile;
pub mod loading;
pub mod selection;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<datafile::DataFile>()
            .init_asset_loader::<datafile::DataFileLoader>()
            .add_plugins(loading::LoadingPlugin)
            .add_systems(OnEnter(Screen::Playing), set_camera_range)
            .init_resource::<selection::SelectedPoint>()
            .add_systems(Update, handle_game_events.run_if(on_event::<GameEvent>));

        app
            .add_systems(Startup, selection::create_marker)
            .add_systems(Update, selection::update_selected_point)
            .add_systems(Update, selection::update_cursor_position);
    }
}

#[derive(Component)]
pub struct LevelLabel;

pub fn handle_game_events(
    mut events: EventReader<GameEvent>,
    asset_server: Res<AssetServer>,
    mut datafile_assets: ResMut<Assets<DataFile>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut commands: Commands,
    level: Option<Single<Entity, With<LevelLabel>>>,
) {
    let level_id = level.map(|s| *s);

    for event in events.read() {
        info!("Handling: {:?}", event);

        match event {
            GameEvent::LoadLevel(level_path) => {
                commands.spawn(new_level(asset_server.load(level_path)));
                next_screen.set(Screen::Loading);
            },
            GameEvent::LoadLevelData(datafile) => {
                let handle = datafile_assets.reserve_handle();
                datafile_assets.insert(handle.id(), datafile.clone());
                commands.spawn(new_level(handle));
                next_screen.set(Screen::Loading);
            },
            GameEvent::LoadingComplete => {
                if let Some(level_id) = level_id {
                    commands.entity(level_id).insert((
                        Visibility::Inherited,
                        StateScoped(Screen::Playing)
                    ));
                }
                next_screen.set(Screen::Playing);
            }
            GameEvent::ExitLevel => {
                next_screen.set(Screen::Title);
            }
        }
    }
}

pub fn set_camera_range(
    terrain: Single<Ref<Terrain>, With<LevelLabel>>,
    mut state: Single<&mut CameraState>,
) {
    let xrange = (terrain.num_blocks[1] * terrain.block_size) as f32;
    let yrange = 1000.0;
    let zrange = (terrain.num_blocks[0] * terrain.block_size) as f32;

    state.focus_range = Vec3::ZERO..Vec3::new(xrange, yrange, zrange);
    state.focus = state.focus_range.start + state.focus_range.end / 2.0;
    state.yaw_range = None;
    state.pitch_range = -TAU/4.0..TAU/4.0;
    state.distance_range = 1.0..xrange.max(zrange).max(1.0);
    state.distance = state.distance_range.end;
    state.mode = CameraMode::Steer;
}
