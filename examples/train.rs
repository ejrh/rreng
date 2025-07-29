use bevy::app::{App, Startup};
use bevy::prelude::Commands;

use rreng::events::GameEvent;
use rreng::RrengPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(RrengPlugin);
    app.add_systems(Startup, load_initial_level);

    app.run();
}

fn load_initial_level(mut commands: Commands) {
    let datafile = ron::from_str(LEVEL_FILE).unwrap();

    commands.send_event(GameEvent::LoadLevelData(datafile));
}

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
