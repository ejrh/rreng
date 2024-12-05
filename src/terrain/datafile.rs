use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;

use crate::terrain::tiles::TileSets;

#[derive(Asset, Clone, Debug, Default, Deserialize, TypePath)]
pub struct DataFile {
    pub bounds: Rect,
    pub size: [usize; 2],
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

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>
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
