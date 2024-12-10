use std::collections::HashMap;
use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;

use crate::terrain::TerrainLayer;
use crate::terrain::tiles::TileSets;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Track {
    pub points: Vec<Vec3>,
}

#[derive(Asset, Clone, Debug, Default, Deserialize, TypePath)]
pub struct DataFile {
    pub size: [usize; 2],
    pub layers: Vec<TerrainLayer>,
    pub bounds: Rect,
    pub tracks: HashMap<String, Track>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DataFileLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not deserialise TOML: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Default)]
pub struct DataFileLoader;

impl AssetLoader for DataFileLoader {
    type Asset = DataFile;
    type Settings = ();
    type Error = DataFileLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>
    ) -> Result<Self::Asset, Self::Error> {
        load_context.load::<TileSets>("data/tiles.toml");

        let mut str = String::new();
        reader.read_to_string(&mut str).await?;
        let datafile = toml::from_str(&str)?;
        Ok(datafile)
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
