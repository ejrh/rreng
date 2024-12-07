use bevy::prelude::*;

use crate::track::segment::{Segment, SegmentLinkage};

#[derive(Component)]
pub struct TrainCar {
    pub segment_id: Entity,
    pub segment_position: f32,
    pub speed: f32,
}

pub fn move_train(
    time: Res<Time>,
    mut trains: Query<&mut TrainCar>,
    segments: Query<(&Segment, &SegmentLinkage), Without<TrainCar>>,
) {
    for mut car in trains.iter_mut() {
        let Ok((segment, linkage)) = segments.get(car.segment_id)
        else { continue };

        if segment.length == 0.0 { continue; }

        if car.speed < 0.0 && car.speed >= -10.0 {
            car.speed -= 0.1 * time.delta_secs();
        } else if car.speed > 0.0 && car.speed <= 10.0 {
            car.speed += 0.1 * time.delta_secs();
        }

        car.segment_position += car.speed * time.delta_secs();
        if car.segment_position >= segment.length {
            if let Some(next_segment) = linkage.next_segment {
                car.segment_id = next_segment;
                car.segment_position -= segment.length;
                debug!("moved onto next segment at {}", car.segment_position);
            } else {
                car.speed = -0.001;
                car.segment_position = segment.length;
                debug!("no next segment, reversing");
            }
        } else if car.segment_position < 0.0 {
            if let Some((prev_segment, prev_length)) = linkage.prev_segment {
                car.segment_id = prev_segment;
                car.segment_position += prev_length;
                debug!("moved onto prev segment at {}", car.segment_position);
            } else {
                car.speed = 0.001;
                car.segment_position = 0.0;
                debug!("no prev segment, reversing");
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
