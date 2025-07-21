use std::io::Write;
use std::path::{Path, PathBuf};

use bevy::log::{error, info};
use bevy::math::Rect;
use thiserror::Error;

use rreng::terrain::tiles::{Tile, TileSet, TileSets};

const TILESETS_PATH: &str = "assets/data/tiles.ron";

#[derive(Error, Debug)]
enum ScanError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tiff(#[from] tiff::TiffError),
}

fn load_tilesets(path: &Path) -> Result<TileSets, std::io::Error> {
    let f = std::fs::File::open(path)?;
    let str = std::io::read_to_string(f)?;
    let tilesets = ron::from_str(&str).unwrap();
    Ok(tilesets)
}

fn save_tilesets(tilesets: &TileSets, path: &Path) -> Result<(), std::io::Error> {
    let mut f = std::fs::File::create(path)?;
    let str = ron::ser::to_string_pretty(&tilesets, ron::ser::PrettyConfig::new()).unwrap();
    f.write_all(str.as_bytes())?;
    Ok(())
}

fn scan_tileset(_name: &str, tileset: &mut TileSet, root_path: &Path) {
    let tiles_glob = root_path.parent().unwrap().join(&tileset.root).join(&tileset.pattern);

    tileset.files.clear();

    info!("Scanning {:?}", tiles_glob);
    for f in glob::glob(tiles_glob.as_os_str().to_str().unwrap()).unwrap() {
        let path = f.unwrap();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        info!("Found {name}");
        if let Ok(tile) = scan_tile(&path, &name) {
            tileset.files.insert(name, tile);
        } else {
            error!("Failed to scan {name}")
        }
    }
}

fn scan_tile(path: &Path, _name: &str) -> Result<Tile, ScanError> {
    let f = std::fs::File::open(path)?;
    let tiff = geotiff::GeoTiff::read(f)?;

    let extent = tiff.model_extent();
    let bounds = Rect::new(
        extent.min().x as f32, extent.min().y as f32,
        extent.max().x as f32, extent.max().y as f32);

    let tile = Tile {
        bounds,
    };

    Ok(tile)
}

fn main() {
    tracing_subscriber::fmt::init();

    let tilesets_filename = PathBuf::from(TILESETS_PATH);
    info!("Scanning tiles in {tilesets_filename:?}");

    let mut tilesets = load_tilesets(&tilesets_filename).unwrap();

    for (name, ts) in tilesets.0.iter_mut() {
        scan_tileset(name, ts, &tilesets_filename);
    }

    save_tilesets(&tilesets, &tilesets_filename).unwrap();
}
