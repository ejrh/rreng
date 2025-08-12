use bevy::prelude::{ChildOf, Cuboid, Entity, Mesh3d, ReflectComponent, Res, ResMut};
use bevy::prelude::ReflectResource;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Changed, Commands, Component, Mesh, Or, Query, Resource, Transform};
use bevy::reflect::Reflect;

use crate::track::segment::Segment;

pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<BridgeRenderParams>()
            .register_type::<Bridge>()
            .init_resource::<BridgeRenderParams>()
            .add_systems(Startup, init_render_params)
            .add_systems(Update, render_bridges);
    }
}

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
pub struct BridgeRenderParams {
    bridge_material: Handle<StandardMaterial>,
    bridge_mesh: Handle<Mesh>,
    pillar_mesh: Handle<Mesh>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bridge {
    pub(crate) pillars: usize,
}

fn init_render_params(
    mut params: ResMut<BridgeRenderParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bridge_material = StandardMaterial::from(Color::srgb(0.2, 0.2, 0.2));
    params.bridge_material = materials.add(bridge_material);

    let bridge_mesh: Mesh = Cuboid::from_size(Vec3::new(5.0, 1.0, 1.0)).into();
    let bridge_mesh = bridge_mesh.translated_by(Vec3::new(0.0, -0.5, 0.5));
    params.bridge_mesh = meshes.add(bridge_mesh);

    let pillar_mesh: Mesh = Cuboid::from_size(Vec3::new(2.0, 1.0, 1.0)).into();
    let pillar_mesh = pillar_mesh.translated_by(Vec3::new(0.0, -0.5, 0.0));
    params.pillar_mesh = meshes.add(pillar_mesh);
}

fn render_bridges(
    bridges: Query<(Entity, &Bridge, &Segment, &Transform), Or<(Changed<Bridge>, Changed<Segment>)>>,
    params: Res<BridgeRenderParams>,
    mut commands: Commands,
) {
    for (seg_id, bridge, seg, seg_transform) in bridges {
        let mut bridge_pos = Transform::default();
        bridge_pos.scale.z = seg.length;
        commands.spawn((
            Mesh3d(params.bridge_mesh.clone()),
            MeshMaterial3d(params.bridge_material.clone()),
            ChildOf(seg_id),
            bridge_pos
        ));

        for i in 0..bridge.pillars {
            let mut pillar_pos = Transform::default();
            pillar_pos.translation = pillar_pos.rotation * Vec3::Z * (i as f32 * seg.length/(bridge.pillars - 1) as f32);
            pillar_pos.scale.y = 10.0;

            commands.spawn((
                Mesh3d(params.pillar_mesh.clone()),
                MeshMaterial3d(params.bridge_material.clone()),
                ChildOf(seg_id),
                pillar_pos
            ));
        }
    }
}
