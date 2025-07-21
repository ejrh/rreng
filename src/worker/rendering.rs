use bevy::asset::{Assets, Handle};
use bevy::color::Srgba;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Added, Capsule3d, ChildOf, Children, Commands, Entity, Mesh, Mesh3d, Query, ResMut, Resource, Transform};
use crate::utils::ConstantApparentSize;
use crate::worker::Worker;

#[derive(Default, Resource)]
pub struct WorkerRenderParams {
    worker_mesh: Handle<Mesh>,
    worker_material: Handle<StandardMaterial>,
}

pub fn setup_render_params(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<WorkerRenderParams>,
) {
    params.worker_mesh = meshes.add(create_worker_mesh());
    params.worker_material = materials.add(StandardMaterial::from_color(Srgba::rgb(0.0, 1.0, 1.0)));
}

pub fn render_workers(
    workers: Query<Entity, Added<Worker>>,
    params: ResMut<WorkerRenderParams>,
    mut commands: Commands,
) {
    for w in workers {
        info!("Rendering worker {:?}", w);
        commands.entity(w).despawn_related::<Children>();
        commands.spawn((
            Mesh3d(params.worker_mesh.clone()),
            MeshMaterial3d(params.worker_material.clone()),
            ConstantApparentSize(10.0..100.0),
            Transform::from_scale(Vec3::splat(1.0)),
            ChildOf(w)
        ));
    }
}

fn create_worker_mesh() -> Mesh {
    const HEIGHT: f32 = 2.0;
    const DIAMETER: f32 = 0.8;

    let mut mesh: Mesh = Capsule3d::new(DIAMETER / 2.0, HEIGHT - DIAMETER).into();
    mesh.translate_by(Vec3::new(0.0, DIAMETER, 0.0));
    mesh
}
