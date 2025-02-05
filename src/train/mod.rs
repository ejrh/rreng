use std::f32::consts::PI;

use bevy::prelude::*;

use crate::track::segment::{Segment, SegmentLinkage};
use crate::train::rendering::{render_trains, setup_render_params, TrainRenderParams};

mod rendering;

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
pub struct TrainCar {
    pub segment_id: Entity,
    pub segment_position: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub length: f32,
}

#[derive(Default)]
pub struct TrainPlugin;

impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<TrainCar>()
            .init_resource::<TrainRenderParams>()
            .add_systems(Startup, setup_render_params)
            .add_systems(Update, render_trains)
            .add_systems(Update, (move_train, update_train_position));
    }
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

        /* Cutoff position of the car where it is turning from/to an adjacent segment */
        let p1 = if linkage.prev_segment.is_some() { car.length / 2.0 } else { 0.0 };
        let p2 = if linkage.next_segment.is_some() { seg.length - car.length / 2.0 } else { seg.length };

        /* Simple case: car is fully within this one segment */
        if (p1..=p2).contains(&car.segment_position) {
            *transform = Transform::default();
            transform.translation.y += crate::track::TRACK_HEIGHT;
            transform.translation.z = car.segment_position;

            *transform = seg_transform.mul_transform(*transform);
            continue;
        }

        /* Complex case: car is mostly on this segment, but part of it is on an adjacent segment.
        First figure out if it's the previous or next segment and set some parameters accordingly.
        Junction is the point where the two segments join; its direction is set along the adjacent
        segment, towards the junction.  Mix is the proportion of the other segment to use for the
        car's rotation. */
        let next_seg: Transform;
        let mut junction: Transform;
        let mix: f32;
        //TODO a hack for 1.0 or -1.0 depending on what end of the train we want to anchor
        let anchor_dir: f32;

        if car.segment_position < p1 {
            let Ok((_, _, prev_seg_transform)) = segments.get(linkage.prev_segment.unwrap().0)
            else { continue; };

            next_seg = *prev_seg_transform;
            junction = *seg_transform;
            junction.rotation = prev_seg_transform.rotation * Quat::from_axis_angle(Vec3::Y, PI);
            mix = 1.0 - car.segment_position / p1;
            anchor_dir = -1.0;
        } else {
            let Ok((_, _, next_seg_transform)) = segments.get(linkage.next_segment.unwrap())
            else { continue; };

            next_seg = *next_seg_transform;
            junction = *next_seg_transform;
            junction.rotation = next_seg_transform.rotation;
            mix = (car.segment_position - p2) / (seg.length - p2);
            anchor_dir = 1.0;
        }

        /* The rotation of the car is interpolated between each segment */
        let car_rotation = seg_transform.rotation.lerp(next_seg.rotation, mix/2.0);

        /* The placement is solved using the triangle sine identity.  Theta is the angle of the
        junction; alpha is the angle between the car and the segment it's mostly on; dist is
        the distance of the opposing point (the one on the adjacent segment) from the junction. */
        let theta = seg_transform.rotation.angle_between(junction.rotation);
        let alpha = seg_transform.rotation.angle_between(car_rotation);
        let dist = (car.length * alpha.sin()) / theta.sin();

        /* connection is where on the adjacent segment we want to connect the train;
        anchor is the part of the train we want to connect there. */
        let connection = junction.translation + junction.back() * dist;
        let anchor = car_rotation * Vec3::Z * (car.length / 2.0) * anchor_dir;

        *transform = Transform::default();
        transform.translation = connection - anchor;
        transform.translation.y += crate::track::TRACK_HEIGHT;
        transform.rotation = car_rotation;
    }
}
