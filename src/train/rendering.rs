use bevy::asset::{AssetServer, Assets, Handle};
use bevy::color::Color;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver, SpotLight};
use bevy::prelude::{Added, ChildOf, Children, Commands, Entity, GltfAssetLabel, LinearRgba, Mesh, Mesh3d, Meshable, Query, Res, ResMut, Resource, Scene, SceneRoot, Sphere, StandardMaterial, Transform};

use crate::train::TrainCar;

#[derive(Default, Resource)]
pub struct TrainRenderParams {
    train_model: Handle<Scene>,
    train_headlight_mesh: Handle<Mesh>,
    train_headlight_material: Handle<StandardMaterial>,
}

pub fn setup_render_params(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<TrainRenderParams>,
) {
    const TRAIN_PATH: &str = "models/lowpoly_train.glb";

    params.train_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset(TRAIN_PATH));

    let mut mesh: Mesh = Sphere::new(0.125).mesh().into();
    mesh.scale_by(Vec3::new(1.0, 1.0, 0.25));
    params.train_headlight_mesh = meshes.add(mesh);
    let mut material = StandardMaterial::from(Color::srgb(1.0, 1.0, 0.75));
    material.emissive = LinearRgba::from(material.base_color);
    params.train_headlight_material = materials.add(material);
}


pub fn render_trains(
    trains: Query<Entity, Added<TrainCar>>,
    params: ResMut<TrainRenderParams>,
    mut commands: Commands,
) {
    for train_id in trains.iter() {
        commands.entity(train_id).despawn_related::<Children>();

        /* Spawn train model and fix up its silly model transform */
        commands.spawn((
            SceneRoot(params.train_model.clone()),
            Transform::default()
                .with_scale(Vec3::splat(3.28084))
                .with_rotation(Quat::from_axis_angle(Vec3::Y, 54.0f32.to_radians())),
            ChildOf(train_id)
        ));

        /* Put some spot lights for the train's headlamps */
        for dir in [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
        ] {
            /* Just one light source in each direction */
            let pos = Vec3::new(0.0, 2.0, 9.0) * dir;
            let target = pos + Vec3::Z * pos.z.signum();
            commands.spawn((
                SpotLight::default(),
                Transform::from_translation(pos).looking_at(target, Vec3::Y),
                ChildOf(train_id),
            ));

            /* A glowing orb on each of the train's headlights */
            for light in [
                Vec3::new(0.84, 1.71, 8.85),
                Vec3::new(-0.9, 1.71, 8.85),
                Vec3::new(-0.02, 2.49, 8.6),
            ] {
                let pos = light * dir;
                commands.spawn((
                    Mesh3d(params.train_headlight_mesh.clone()),
                    MeshMaterial3d(params.train_headlight_material.clone()),
                    NotShadowCaster,
                    NotShadowReceiver,
                    Transform::from_translation(pos),
                    ChildOf(train_id),
                ));
            }
        }
    }
}
