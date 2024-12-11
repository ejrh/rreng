use bevy::prelude::*;

use crate::track::segment::{Segment, SegmentLinkage};

#[derive(Component, Reflect)]
pub struct TrainCar {
    pub segment_id: Entity,
    pub segment_position: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub length: f32,
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

        if car.speed < 0.0 && car.speed >= -car.max_speed {
            car.speed -= car.acceleration * time.delta_secs();
        } else if car.speed > 0.0 && car.speed <= car.max_speed {
            car.speed += car.acceleration * time.delta_secs();
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
    segments: Query<(&Segment, &SegmentLinkage, &Transform), Without<TrainCar>>,
) {
    for (car, mut transform) in trains.iter_mut() {
        let Ok((seg, linkage, seg_transform)) = segments.get(car.segment_id)
        else { continue; };

        let p1 = car.length / 2.0;
        let p2 = seg.length - car.length / 2.0;

        *transform = Transform::default();
        transform.translation.z = car.segment_position;
        transform.translation.y += crate::track::TRACK_HEIGHT;

        *transform = seg_transform.mul_transform(*transform);

        if linkage.next_segment.is_some() && car.segment_position > p2 {
            let Ok((_, _, next_seg_transform)) = segments.get(linkage.next_segment.unwrap())
            else { continue; };

            let mix = (car.segment_position - p2) / (seg.length - p2);

            transform.rotation = seg_transform.rotation.lerp(next_seg_transform.rotation, mix/2.0);
        }

        if linkage.prev_segment.is_some() && car.segment_position < p1 {
            let Ok((_, _, prev_seg_transform)) = segments.get(linkage.prev_segment.unwrap().0)
            else { continue; };

            let mix = 1.0 - car.segment_position / p1;

            transform.rotation = seg_transform.rotation.lerp(prev_seg_transform.rotation, mix/2.0);
        }
    }
}
