use bevy::prelude::*;

#[derive(Default)]
pub(crate) struct SkyPlugin {
}

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(create_lights);
   }
}

fn create_lights(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
