use std::io::Cursor;

use bevy::asset::{Asset, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;
use tiff::decoder::DecodingResult;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Chunk {
    pub(crate) position: (isize, isize),
    pub(crate) elevation: String,
}

#[derive(Asset, Clone, Debug, Default, Deserialize, TypePath)]
pub struct DataFile {
    pub(crate) chunk_dimensions: (isize, isize),
    pub(crate) chunks: Vec<Chunk>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DataFileLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not deserialise JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Default)]
pub struct DataFileLoader;

impl AssetLoader for DataFileLoader {
    type Asset = DataFile;
    type Settings = ();
    type Error = DataFileLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut str = String::new();
            reader.read_to_string(&mut str).await?;
            let datafile = serde_json::from_str(&str)?;
            Ok(datafile)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Asset, Debug, TypePath)]
pub struct ChunkElevation {
    pub(crate) heights: Vec<Vec<f32>>,
}

#[derive(Default)]
pub struct ChunkElevationLoader;

impl AssetLoader for ChunkElevationLoader {
    type Asset = ChunkElevation;
    type Settings = ();
    type Error = DataFileLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let chunk_elevation = decode_elevation(&bytes)?;
            Ok(chunk_elevation)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tif"]
    }
}

pub fn decode_elevation(bytes: &[u8]) -> std::io::Result<ChunkElevation> {
    let cursor = Cursor::new(bytes);
    let mut decoder = tiff::decoder::Decoder::new(cursor).unwrap();

    let dims = decoder.dimensions().unwrap();
    let width = dims.0 as usize;
    let height = dims.1 as usize;

    let im = decoder.read_image().unwrap();
    let DecodingResult::F32(raw_data) = im else { panic!() };

    let mut data = Vec::new();
    for i in 0..height {
        data.push(Vec::new());
        for j in 0..width {
            data[i].push(raw_data[i * width + j]);
        }
    }

    Ok(ChunkElevation { heights: data })
}
