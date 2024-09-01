use std::f32::consts::TAU;

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

const SUN_RADIUS: f32 = 700.0;
const SUN_DISTANCE: f32 = 149000.0;
const SUN_COLOUR: Color = Color::srgb(1.0, 1.0, 0.75);
const SECONDS_PER_DAY: f32 = 1000.0;

fn create_lights(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 50.0,
    });

    commands
        .spawn(Sun { angle: 0.0 })
        .insert({
            let sun_mesh: Mesh = Sphere::new(SUN_RADIUS).into();
            let mut sun_material = StandardMaterial::from(SUN_COLOUR);
            sun_material.unlit = true;
            MaterialMeshBundle {
                mesh: meshes.add(sun_mesh),
                material: materials.add(sun_material),
                transform: Transform::from_xyz(SUN_DISTANCE, 0.0, 0.0),
                ..default()
            }})
        .insert((NotShadowCaster, NotShadowReceiver))
        .with_children(|cb| {
            cb.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 2500.0,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::default().looking_at(-Vec3::X, Vec3::Y),
                ..default()
            });
        });
}

fn move_sun(
    time: Res<Time>,
    mut sun_query: Query<(&mut Sun, &mut Transform)>,
    mut light_query: Query<&mut DirectionalLight>,
) {
    let (mut sun, mut transform) = sun_query.single_mut();

    sun.angle += TAU/SECONDS_PER_DAY * time.delta_seconds();
    transform.rotation = Quat::from_axis_angle(Vec3::Z, sun.angle);
    transform.translation = transform.rotation.mul_vec3(Vec3::new(SUN_DISTANCE, 0.0, 0.0));

    let altitude = transform.translation.y / SUN_DISTANCE;

    let mut light = light_query.single_mut();

    if altitude > 0.5 {
        light.color = Color::WHITE;
    } else if altitude > 0.1 {
        light.color = Color::srgb(1.0, altitude*2.0, altitude*2.0)
    } else if altitude > 0.0 {
        light.color = Color::srgb(altitude*10.0, altitude*2.0, altitude*2.0)
    } else {
        light.color = Color::BLACK;
    }
}
