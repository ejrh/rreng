use bevy::prelude::*;
use crate::track::segment::Segment;

const TRAIN_PATH: &str = "models/lowpoly_train.glb";

#[derive(Component)]
pub struct TrainCar {
    segment_id: Entity,
    segment_position: f32,
}

pub fn update_train_position(
    mut trains: Query<(&TrainCar, &mut Transform)>,
    segments: Query<(&Segment, &Transform), Without<TrainCar>>,
) {
    for (car, mut transform) in trains.iter_mut() {
        let (segment, seg_transform) = segments.get(car.segment_id).unwrap();

        *transform = seg_transform.clone();
        transform.translation.y += crate::track::TRACK_HEIGHT;

        /* Fix up silly model transform */
        transform.scale = Vec3::splat(3.28084);
        transform.rotate(Quat::from_axis_angle(Vec3::Y, 54.0f32.to_radians()));
    }
}

pub fn create_train(
    asset_server: Res<AssetServer>,
    all_segments: Query<Entity, With<Segment>>,
    mut commands: Commands,
) {
    let first_segment_id = all_segments.iter().next().unwrap();

    commands.spawn((
        TrainCar {
            segment_id: first_segment_id,
            segment_position: 0.0,
        },
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(TRAIN_PATH))),
    ));
}
