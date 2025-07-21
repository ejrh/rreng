use bevy::app::{App, Plugin, PluginGroup, Update};
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::DefaultPlugins;
use bevy::prelude::{default, WindowPlugin};

pub mod camera;
pub mod debug;
pub mod events;
pub mod level;
pub mod sky;
pub mod terrain;
pub mod tools;
pub mod track;
pub mod train;
pub mod ui;
pub mod utils;
pub mod worker;

pub struct RrengPlugin;

impl Plugin for RrengPlugin {
    fn build(&self, app: &mut App) {
        let window_plugin = pick_window_plugin();

        app
            .add_plugins(
                DefaultPlugins
                    .set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() })
                    .set(window_plugin)
            ).add_plugins(camera::CameraPlugin)
            .add_plugins(sky::SkyPlugin)
            .add_plugins(level::LevelPlugin)
            .add_plugins(terrain::TerrainPlugin)
            .add_plugins(track::TrackPlugin)
            .add_plugins(train::TrainPlugin)
            .add_plugins(worker::WorkerPlugin)
            .add_plugins(debug::DebugPlugin)
            .add_systems(Update, utils::fix_apparent_size)
            .add_event::<events::GraphicsEvent>();

        /* Outside web mode, quit on ESC being pressed */
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, utils::close_on_esc);
    }
}

fn pick_window_plugin() -> WindowPlugin {
    let on_web = cfg!(target_arch = "wasm32");

    if on_web {
        /* In web mode, fill the whole canvas */
        use bevy::window::Window;
        WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }
    } else {
        WindowPlugin::default()
    }
}
