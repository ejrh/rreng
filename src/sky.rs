use std::f32::consts::{TAU};

use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

#[derive(Default)]
pub struct SkyPlugin {
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

const SUN_RADIUS: f32 = 200.0;
const SUN_DISTANCE: f32 = 0.0;

fn create_lights(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });

    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 2500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(Sun { angle: 0.0 })
        .with_children(|cb| {
            cb
                .spawn({
                    let sun_mesh: Mesh = Sphere::new(SUN_RADIUS).into();
                    let mut sun_material = StandardMaterial::from(Color::srgb(1.0, 1.0, 0.75));
                    sun_material.unlit = true;
                    MaterialMeshBundle {
                        mesh: meshes.add(sun_mesh),
                        material: materials.add(sun_material),
                        transform: Transform::from_xyz(SUN_DISTANCE, 0.0, 0.0),
                        ..default()
                    }})
                .insert((NotShadowCaster, NotShadowReceiver));
        });
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
        light.color = Color::srgb(1.0, altitude*2.0, altitude*2.0)
    } else if altitude > 0.0 {
        light.color = Color::srgb(altitude*10.0, altitude*2.0, altitude*2.0)
    } else {
        light.color = Color::BLACK;
    }
    let where_in_sky = where_in_sky * SUN_DISTANCE;
    *transform = Transform::from_translation(where_in_sky).looking_at(Vec3::ZERO, Vec3::Y);
}
