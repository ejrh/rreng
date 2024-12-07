use std::collections::HashMap;
use std::io::Cursor;

use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::math::Rect;
use bevy::prelude::TypePath;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tiff::decoder::DecodingResult;
use crate::terrain::TerrainLayer;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tile {
    pub bounds: Rect,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TileSet {
    pub chunk_dimensions: (usize, usize),
    pub root: String,
    pub pattern: String,
    pub layer: TerrainLayer,
    pub files: HashMap<String, Tile>,
}

#[derive(Asset, Debug, Deserialize, Serialize, TypePath)]
pub struct TileSets(pub HashMap<String, TileSet>);

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TileSetsLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not deserialise TOML: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Default)]
pub struct TileSetsLoader;

impl AssetLoader for TileSetsLoader {
    type Asset = TileSets;
    type Settings = ();
    type Error = TileSetsLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>
    ) -> Result<Self::Asset, Self::Error> {
        let mut str = String::new();
        reader.read_to_string(&mut str).await?;
        let tilesets = toml::from_str(&str)?;
        Ok(tilesets)
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ElevationFileLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not decode TIF: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Asset, Debug, TypePath)]
pub struct ElevationFile {
    pub(crate) heights: ndarray::Array2<f32>,
}

#[derive(Default)]
pub struct ElevationFileLoader;

impl AssetLoader for ElevationFileLoader {
    type Asset = ElevationFile;
    type Settings = ();
    type Error = ElevationFileLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let elevation_file = decode_elevation(&bytes)?;
        Ok(elevation_file)
    }

    fn extensions(&self) -> &[&str] {
        &["tif"]
    }
}

pub fn decode_elevation(bytes: &[u8]) -> Result<ElevationFile, ElevationFileLoaderError> {
    let cursor = Cursor::new(bytes);
    let mut decoder = tiff::decoder::Decoder::new(cursor).unwrap();

    let dims = decoder.dimensions().unwrap();
    let width = dims.0 as usize;
    let height = dims.1 as usize;

    let im = decoder.read_image().unwrap();
    let DecodingResult::F32(raw_data) = im else { panic!() };

    let mut data = ndarray::Array2::zeros((height, width));

    for (r, chunk) in raw_data.chunks_exact(width).enumerate() {
        let mut row_view = data.row_mut(r);
        if let Some(dest) = row_view.as_slice_mut() {
            dest.copy_from_slice(chunk);
        }
    }

    Ok(ElevationFile { heights: data })
}
