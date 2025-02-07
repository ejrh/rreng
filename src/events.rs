use bevy::prelude::Event;

#[derive(Debug, Event)]
pub enum GraphicsEvent {
    LoadLevel,
    RenderTerrain,
    MoveCamera,
}
