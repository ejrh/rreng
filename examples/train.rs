use bevy::app::{App, Startup};
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::ResMut;

use rreng::RrengPlugin;
use rreng::level::datafile::DataFile;
use rreng::level::loading::LoadingState;

fn main() {
    let mut app = App::new();

    app.add_plugins(RrengPlugin);
    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut loading_state: ResMut<LoadingState>,
    mut datafile_assets: ResMut<Assets<DataFile>>,
) {
    let datafile = ron::from_str(LEVEL_FILE).unwrap();
    datafile_assets.insert(LEVEL_FILE_HANDLE.id(), datafile);
    loading_state.datafile_handle = LEVEL_FILE_HANDLE;
}

pub const LEVEL_FILE_HANDLE: Handle<DataFile> = Handle::weak_from_u128(19781219_1);

const LEVEL_FILE: &str = r#"
DataFile(
    size: (64, 64),
    layers: [ Elevation ],

    bounds: (
        min: (1749280.0, 5429860.0),
        max: (1749760.0, 5430580.0)
    ),

    tracks: {
        "JVL": (
            points: [
                (4.0, 2.5, 4.0),
                (36.0, 2.2, 28.0),
                (60.0, 2.2, 60.0),
            ],
        ),
    }
)
"#;
