use bevy::prelude::OnEnter;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::AssetApp;
use bevy::prelude::{on_event, resource_changed, IntoScheduleConfigs};

use crate::events::GameEvent;
use crate::level::loading::set_camera_range;
use crate::screens::Screen;

pub mod datafile;
pub mod loading;
pub mod selection;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<datafile::DataFile>()
            .init_asset_loader::<datafile::DataFileLoader>()
            .init_resource::<loading::LoadingState>()
            .add_systems(Update, loading::tilesets_loaded)
            .add_systems(Update, loading::elevation_loaded)
            .add_systems(Update, loading::datafile_loaded)
            .add_systems(Update, loading::check_loading_state.run_if(resource_changed::<loading::LoadingState>))
            .add_systems(OnEnter(Screen::Playing), set_camera_range)
            .init_resource::<selection::SelectedPoint>()
            .add_systems(Update, loading::handle_game_events.run_if(on_event::<GameEvent>));

        app
            .add_systems(Startup, selection::create_marker)
            .add_systems(Update, selection::update_selected_point)
            .add_systems(Update, selection::update_cursor_position);
    }
}
