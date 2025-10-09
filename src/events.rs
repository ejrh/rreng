use bevy::ecs::message::Message;

#[derive(Debug, Message)]
pub enum GameMessage {
    LoadLevel(String),
    LoadLevelData(crate::level::datafile::DataFile),
    LoadingComplete,
    ExitLevel,
}

#[derive(Debug, Message)]
pub enum GraphicsMessage {
    LoadedLevel,
    RenderTerrain,
    MoveCamera,
}
