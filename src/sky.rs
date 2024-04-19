use std::f32::consts::{TAU};
use bevy::prelude::*;

#[derive(Default)]
pub(crate) struct SkyPlugin {
}

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, create_lights)
        .add_systems(Update, move_sun);
   }
}

#[derive(Component)]
struct Sun {
    angle: f32
}

fn create_lights(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(Sun { angle: 0.0 });
}

fn move_sun(time: Res<Time>, mut query: Query<(&mut Sun, &mut Transform, &mut DirectionalLight)>) {
    let (mut sun, mut transform, mut light) = query.single_mut();

    sun.angle += TAU/100.0 * time.delta_seconds();
    let where_in_sky = Quat::from_axis_angle(Vec3::Z, sun.angle);
    let where_in_sky= where_in_sky.mul_vec3(Vec3::X);
    let altitude = where_in_sky.dot(Vec3::Y);

    if altitude > 0.5 {
        light.color = Color::WHITE;
    } else if altitude > 0.1 {
        light.color = Color::rgb(1.0, altitude*2.0, altitude*2.0)
    } else if altitude > 0.0 {
        light.color = Color::rgb(altitude*10.0, altitude*2.0, altitude*2.0)
    } else {
        light.color = Color::BLACK;
    }
    let where_in_sky = where_in_sky * 200.0;
    *transform = Transform::from_translation(where_in_sky).looking_at(Vec3::ZERO, Vec3::Y);
}
