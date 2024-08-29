use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Default)]
pub struct DebugPlugin;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum DebugState {
    #[default]
    Off,
    On,
}

impl DebugState {
    fn inverse(&self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<DebugState>()
            .add_plugins(WorldInspectorPlugin::new().run_if(in_state(DebugState::On)))
            .add_systems(Update, toggle_debug);

    }
}

fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<DebugState>>,
    mut next_state: ResMut<NextState<DebugState>>,
) {
    if keys.just_pressed(KeyCode::F5) {
        next_state.set(current_state.inverse());
    }
}
