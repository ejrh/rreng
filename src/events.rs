use bevy::asset::AssetId;
use bevy::prelude::Event;

use crate::terrain::tiles::ElevationFile;

#[derive(Debug, Event)]
pub enum DataEvent {
    LoadedElevation(AssetId<ElevationFile>),
}

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
