use bevy::prelude::Event;

#[derive(Debug, Event)]
pub enum GameEvent {
    LoadLevel(String),
    LoadLevelData(crate::level::datafile::DataFile),
    LoadingComplete,
    ExitLevel,
}

#[derive(Debug, Event)]
pub enum GraphicsEvent {
    LoadedLevel,
    RenderTerrain,
    MoveCamera,
}
