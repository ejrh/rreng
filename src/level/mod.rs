use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::AssetApp;
use bevy::prelude::{resource_changed, IntoScheduleConfigs};

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
            .add_systems(Startup, loading::load_initial_level)
            .add_systems(Update, loading::tilesets_loaded)
            .add_systems(Update, loading::elevation_loaded)
            .add_systems(Update, loading::datafile_loaded)
            .add_systems(Update, (loading::check_loading_state, loading::set_camera_range)
                .run_if(resource_changed::<loading::LoadingState>))
            .init_resource::<selection::SelectedPoint>();

        app
            .add_systems(Startup, selection::create_marker)
            .add_systems(Update, selection::update_selected_point)
            .add_systems(Update, selection::update_cursor_position)
            .add_systems(Startup, selection::create_cursor_position_text);
    }
}
