use std::f32::consts::TAU;

use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<Sun>()
        .add_systems(Startup, create_lights)
        .add_systems(Update, move_sun);
   }
}

#[derive(Reflect, Component)]
struct Sun {
    angle: f32,
    period: f32,
}

const SUN_RADIUS: f32 = 700.0;
const SUN_DISTANCE: f32 = 149000.0;
const SUN_COLOUR: Color = Color::srgb(1.0, 1.0, 0.75);
const INITIAL_SUN_ANGLE: f32 = TAU/8.0;
const SUN_PERIOD: f32 = 3600.0;

fn create_lights(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 50.0,
    });

    let sun_mesh: Mesh = Sphere::new(SUN_RADIUS).into();
    let mut sun_material = StandardMaterial::from(SUN_COLOUR);
    sun_material.unlit = true;

    commands
        .spawn((
            Sun {
                angle: INITIAL_SUN_ANGLE,
                period: SUN_PERIOD,
            },
            Mesh3d(meshes.add(sun_mesh)),
            MeshMaterial3d(materials.add(sun_material)),
            Transform::from_xyz(SUN_DISTANCE, 0.0, 0.0),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .with_children(|cb| {
            cb.spawn((
                DirectionalLight {
                    illuminance: 2500.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::default().looking_at(-Vec3::X, Vec3::Y),
            ));
        });
}

fn move_sun(
    time: Res<Time>,
    mut sun_query: Query<(&mut Sun, &mut Transform)>,
    mut light_query: Query<&mut DirectionalLight>,
) {
    let (mut sun, mut transform) = sun_query.single_mut();

    sun.angle += TAU/sun.period * time.delta_secs();
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
