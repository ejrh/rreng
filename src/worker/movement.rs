use bevy::math::{Vec3, Vec3Swizzles};
use bevy::prelude::{Query, Res, Time, Transform};

use crate::terrain::{Terrain, TerrainData};
use crate::worker::Worker;

pub fn move_workers(
    time: Res<Time>,
    terrain: Res<Terrain>,
    terrain_data: Res<TerrainData>,
    mut workers: Query<(&mut Worker, &mut Transform)>,
) {
    for (mut w, mut wt) in workers.iter_mut() {
        wt.translation += w.velocity * time.delta_secs();
        wt.translation = wt.translation.clamp(Vec3::ZERO, Vec3::new(terrain.size[0] as f32, 0.0, terrain.size[1] as f32));
        wt.translation.y = terrain_data.elevation_at(wt.translation.xz());

        let thrust = w.target_velocity.normalize_or_zero() * w.acceleration;
        let thrust = Vec3::new(thrust.x, 0.0, thrust.y);
        w.velocity += thrust * time.delta_secs();

        let friction = w.velocity * 0.2;
        w.velocity -= friction * time.delta_secs();
    }
}
