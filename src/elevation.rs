use std::{
    collections::HashMap,
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
    pub fn new(path: &Path) -> Result<Self, ElevationError> {
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

    pub fn cached(&self) -> CachedElevationMap<'_> {
        CachedElevationMap {
            map: self,
            cache: HashMap::new(),
        }
    }

    /// looks up the elevation for the given coordinates, or returns 0 if there's an error
    #[allow(dead_code)]
    pub fn lookup_or_0(&self, lng: f64, lat: f64) -> i32 {
        self.lookup(lng, lat).unwrap_or(0)
    }

    /// looks up the elevation for the given coordinates
    #[allow(dead_code)]
    pub fn lookup(&self, lng: f64, lat: f64) -> Result<i32, ElevationError> {
        let path = self
            .tree
            .search(Rect::new_point([lng, lat]))
            .next()
            .ok_or(ElevationError::CoordNotFound)?
            .data;

        let data = Dataset::open(path)?;

        lookup(&data, lng, lat)
    }
}

pub struct CachedElevationMap<'a> {
    map: &'a ElevationMap,
    cache: HashMap<PathBuf, Dataset>,
}

impl<'a> CachedElevationMap<'a> {
    /// looks up the elevation for the given coordinates, or returns 0 if there's an error
    pub fn lookup_or_0(&mut self, lng: f64, lat: f64) -> i32 {
        self.lookup(lng, lat).unwrap_or(0)
    }

    /// looks up the elevation for the given coordinates
    pub fn lookup(&mut self, lng: f64, lat: f64) -> Result<i32, ElevationError> {
        let path = self
            .map
            .tree
            .search(Rect::new_point([lng, lat]))
            .next()
            .ok_or(ElevationError::CoordNotFound)?
            .data;

        if let Some(data) = self.cache.get(path) {
            lookup(data, lng, lat)
        } else {
            let data = Dataset::open(path)?;
            let r = lookup(&data, lng, lat);
            self.cache.insert(path.clone(), data);

            r
        }
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
fn lookup(data: &Dataset, lng: f64, lat: f64) -> Result<i32, ElevationError> {
    let gt = data.geo_transform()?;

    // unsure where this came from. i found it somewhere but i cant find it again
    let x = ((lng - gt[0]) / gt[1]) as i64;
    let y = ((lat - gt[3]) / gt[5]) as i64;

    let band = data.rasterband(1)?;

    let point_array = band.read_as_array::<i32>(
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
