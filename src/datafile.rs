use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize};
use tiff::decoder::DecodingResult;

#[derive(Debug, Deserialize)]
pub(crate) struct Chunk {
    pub(crate) position: (isize, isize),
    pub(crate) elevation: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DataFile {
    pub(crate) chunk_dimensions: (isize, isize),
    pub(crate) chunks: Vec<Chunk>,
}

pub(crate) fn load_datafile<P: AsRef<Path>>(filename: P) -> std::io::Result<DataFile> {
    let mut file = File::open(filename)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;
    let datafile: DataFile = serde_json::from_str(json.as_str())?;

    Ok(datafile)
}

pub fn load_elevation<P: AsRef<Path>>(filename: P) -> std::io::Result<Vec<Vec<f32>>> {
    let file = File::open(filename)?;
    let mut decoder = tiff::decoder::Decoder::new(file).unwrap();

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

    Ok(data)
}
