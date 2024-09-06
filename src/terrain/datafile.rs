use std::io::Cursor;

use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
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

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>
    ) -> Result<Self::Asset, Self::Error> {
        let mut str = String::new();
        reader.read_to_string(&mut str).await?;
        let datafile = serde_json::from_str(&str)?;
        Ok(datafile)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
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
    type Error = DataFileLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>
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

pub fn decode_elevation(bytes: &[u8]) -> std::io::Result<ElevationFile> {
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
