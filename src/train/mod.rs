use bevy::prelude::*;

use crate::track::segment::Segment;

const TRAIN_PATH: &str = "models/lowpoly_train.glb";

#[derive(Component)]
pub struct TrainCar {
    segment_id: Entity,
    segment_position: f32,
    speed: f32,
}

pub fn move_train(
    time: Res<Time>,
    mut trains: Query<&mut TrainCar>,
    segments: Query<&Segment, Without<TrainCar>>,
) {
    for mut car in trains.iter_mut() {
        let segment = segments.get(car.segment_id).unwrap();
        if segment.length == 0.0 { continue; }

        if car.speed < 0.0 {
            if car.speed >= -10.0 {
                car.speed -= 0.1 * time.delta_secs();
            }
        } else {
            if car.speed <= 10.0 {
                car.speed += 0.1 * time.delta_secs();
            }
        }

        car.segment_position += car.speed * time.delta_secs();
        if car.segment_position >= segment.length {
            let next_segment = segment.next_segment;
            if next_segment.index() != 0 {
                car.segment_id = next_segment;
                car.segment_position -= segment.length;
                info!("moved onto next segment at {}", car.segment_position);
            } else {
                car.speed = -0.001;
                car.segment_position = segment.length;
                info!("no next segment, reversing");
            }
        } else if car.segment_position < 0.0 {
            let prev_segment = segment.prev_segment;
            if prev_segment.index() != 0 {
                car.segment_id = prev_segment;
                car.segment_position += segment.prev_length;
                info!("moved onto prev segment at {}", car.segment_position);
            } else {
                car.speed = 0.001;
                car.segment_position = 0.0;
                info!("no prev segment, reversing");
            }
        }
    }
}

pub fn update_train_position(
    mut trains: Query<(&TrainCar, &mut Transform)>,
    segments: Query<&Transform, (With<Segment>, Without<TrainCar>)>,
) {
    for (car, mut transform) in trains.iter_mut() {
        let Ok(seg_transform) = segments.get(car.segment_id)
        else { continue; };

        *transform = Transform::default();
        transform.translation.z = car.segment_position;
        transform.translation.y += crate::track::TRACK_HEIGHT;

        /* Fix up silly model transform */
        transform.scale = Vec3::splat(3.28084);
        transform.rotate(Quat::from_axis_angle(Vec3::Y, 54.0f32.to_radians()));

        *transform = seg_transform.mul_transform(*transform);
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
            segment_position: 10.0,
            speed: 0.0,
        },
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(TRAIN_PATH))),
    ));
}
