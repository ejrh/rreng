use std::time::Duration;

use bevy::log::info;
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::{Query, Res, Single, Time, Transform, With};
use rand::Rng;
use crate::level::LevelLabel;
use crate::terrain::{Terrain, TerrainData};
use crate::worker::{Behaviour, Worker};

pub fn update_workers(
    time: Res<Time>,
    level: Single<(&Terrain, &TerrainData), With<LevelLabel>>,
    mut workers: Query<(&mut Worker, &mut Behaviour, &Transform)>,
) {
    let (terrain, terrain_data) = *level;
    
    for (mut w, mut b, wt) in workers.iter_mut() {
        match *b {
            Behaviour::Idle => {
                if time.elapsed() - w.behaviour_since > Duration::from_secs_f32(10.0) {
                    let mut rng = rand::thread_rng();
                    let mut target = Vec3::new(
                        rng.gen_range(0.0..(terrain.size[1] as f32)),
                        0.0,
                        rng.gen_range(0.0..(terrain.size[0] as f32))
                    );
                    target.y = terrain_data.elevation_at(target.xz());
                    *b = Behaviour::WalkingTo(target);
                    w.behaviour_since = time.elapsed();
                    info!("Set target {}", target);
                }
            },
            Behaviour::WalkingTo(target) => {
                w.target_velocity = (target - wt.translation).xz();
                if w.target_velocity.length_squared() < 1.0 {
                    w.target_velocity = Vec2::ZERO;
                    *b = Behaviour::Idle;
                    w.behaviour_since = time.elapsed();
                    info!("Reached target");
                }
            }
        }
    }
}
