use bevy::prelude::*;

const TRAIN_PATH: &str = "models/lowpoly_train.glb";

pub fn create_train(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let x = 480.0;
    let z = 1770.0;
    let y = 216.5;
    let point = Vec3::new(x, y + 0.15, z);

    let angle = Quat::from_axis_angle(Vec3::Y, (54.0f32).to_radians());

    commands.spawn(SceneBundle {
        scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(TRAIN_PATH)),
        transform: Transform::from_scale(Vec3::splat(3.28084)).with_rotation(angle).with_translation(point),
        ..default()
    });
}
