use std::path::PathBuf;

use ahash::HashMap;
use gdal::Dataset;

use super::ElevationMap;

pub struct CachedGdalElevationMap<'a> {
    map: &'a dyn ElevationMap,
    cache: HashMap<PathBuf, Dataset>,
}

impl<'a> ElevationMap for CachedGdalElevationMap<'a> {
    /// looks up the elevation for the given coordinates
    fn lookup(&mut self, lng: f64, lat: f64) -> Result<i32, ElevationError> {
        let path = self.map.dataset_path(lng, lat)?;

        if let Some(data) = self.cache.get(path) {
            lookup(data, lng, lat)
        } else {
            let data = Dataset::open(path)?;
            let r = lookup(&data, lng, lat);
            self.cache.insert(path.to_path_buf(), data);

            r
        }
    }
}
