use std::{
    io,
    path::{Path, PathBuf},
};

use gdal::{raster::ResampleAlg, Dataset};
use rtree_rs::{RTree, Rect};
use thiserror::Error;

pub struct ElevationMap {
    tree: RTree<2, f64, PathBuf>,
}

impl ElevationMap {
    pub fn new() -> Result<Self, ElevationError> {
        let path = Path::new("./data/elevation");

        let mut tree = RTree::new();

        for entry in path.read_dir()?.flatten() {
            if entry.file_name().to_string_lossy().ends_with(".tif") {
                let corner_coords = get_coorner_coords(&entry.path())?;
                tree.insert(
                    Rect::new(corner_coords.bottom_left, corner_coords.top_right),
                    entry.path(),
                );
            }
        }

        Ok(Self { tree })
    }

    /// looks up the elevation for the given coordinates
    pub fn lookup(&self, lng: f64, lat: f64) -> Result<i64, ElevationError> {
        let path = self
            .tree
            .search(Rect::new_point([lng, lat]))
            .next()
            .ok_or(ElevationError::CoordNotFound)?
            .data;

        // TODO cache the dataset
        let data = Dataset::open(path)?;

        lookup(&data, lng, lat)
    }
}

fn get_coorner_coords(path: &Path) -> Result<CornerCoords, ElevationError> {
    let data = Dataset::open(path)?;
    let [ulx, xres, _xskew, uly, _yskew, yres] = data.geo_transform()?;

    let (xsize, ysize) = data.raster_size();

    let lrx = ulx + (xsize as f64 * xres);
    let lry = uly + (ysize as f64 * yres);

    Ok(CornerCoords {
        // top_left: (ulx, uly),
        top_right: [lrx, uly],
        bottom_left: [ulx, lry],
        // bottom_right: (lrx, lry),
    })
}

#[derive(Debug)]
struct CornerCoords {
    // top_left: (f64, f64),
    top_right: [f64; 2],
    bottom_left: [f64; 2],
    // bottom_right: (f64, f64),
}

// https://stackoverflow.com/questions/13439357/extract-point-from-raster-in-gdal
fn lookup(data: &Dataset, lng: f64, lat: f64) -> Result<i64, ElevationError> {
    let gt = data.geo_transform()?;

    // unsure where this came from. i found it somewhere but i cant find it again
    let x = ((lng - gt[0]) / gt[1]) as i64;
    let y = ((lat - gt[3]) / gt[5]) as i64;

    let band = data.rasterband(1)?;

    let point_array = band.read_as_array::<i64>(
        (x as isize, y as isize),
        (1, 1),
        (1, 1),
        Some(ResampleAlg::NearestNeighbour),
    )?;

    Ok(point_array.into_raw_vec()[0])
}

#[derive(Error, Debug)]
pub enum ElevationError {
    #[error("io error")]
    Network(#[from] io::Error),
    #[error("gdal error")]
    Gdal(#[from] gdal::errors::GdalError),
    #[error("coordinate not found")]
    CoordNotFound,
}
