use bevy::asset::{AssetServer, Handle};
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::math::{Quat, Vec3};
use bevy::pbr::SpotLight;
use bevy::prelude::{Added, Commands, Entity, GltfAssetLabel, Query, Res, ResMut, Resource, Scene, SceneRoot, Transform};

use crate::train::TrainCar;

#[derive(Default, Resource)]
pub struct TrainRenderParams {
    train_model: Handle<Scene>,
}

pub fn setup_render_params(
    asset_server: Res<AssetServer>,
    mut params: ResMut<TrainRenderParams>,
) {
    const TRAIN_PATH: &str = "models/lowpoly_train.glb";

    params.train_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset(TRAIN_PATH));
}


pub fn render_trains(
    trains: Query<Entity, Added<TrainCar>>,
    params: ResMut<TrainRenderParams>,
    mut commands: Commands,
) {
    for train_id in trains.iter() {
        commands.entity(train_id).despawn_descendants();

        /* Spawn train model and fix up its silly model transform */
        commands.spawn((
            SceneRoot(params.train_model.clone()),
            Transform::default()
                .with_scale(Vec3::splat(3.28084))
                .with_rotation(Quat::from_axis_angle(Vec3::Y, 54.0f32.to_radians())),
        )).set_parent(train_id);

        /* Put some spot lights for the train's headlamps */
        const LIGHT_POSITION: Vec3 = Vec3::new(0.85, 1.7, 9.3);
        for (xs, zs) in [
            (-1.0, -1.0),
            (-1.0, 1.0),
            (1.0, -1.0),
            (1.0, 1.0),
        ] {
            let pos = LIGHT_POSITION * Vec3::new(xs, 1.0, zs);
            let target = (LIGHT_POSITION + Vec3::new(0.0, 0.0, 10.0)) * Vec3::new(xs, 1.0, zs);
            commands.spawn((
                SpotLight::default(),
                Transform::from_translation(pos).looking_at(target, Vec3::Y),
            )).set_parent(train_id);
        }
    }
}
