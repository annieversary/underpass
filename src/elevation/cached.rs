use std::{path::PathBuf, sync::Arc};

use ahash::HashMap;
use gdal::Dataset;

use super::{
    gdal::{lookup_in_dataset, GdalElevationMap},
    ElevationError,
};

pub(super) struct CachedGdalElevationMap {
    map: GdalElevationMap,
    // cache: HashMap<PathBuf, Arc<Dataset>>,
}

impl CachedGdalElevationMap {
    pub(super) fn new(map: GdalElevationMap) -> Self {
        Self {
            map,
            // cache: Default::default(),
        }
    }

    /// looks up the elevation for the given coordinates
    pub(super) async fn lookup(&self, lng: f64, lat: f64) -> Result<i32, ElevationError> {
        let path = self.map.dataset_path(lng, lat)?;

        let data = Dataset::open(path)?;
        lookup_in_dataset(&data, lng, lat)
        // if let Some(data) = self.cache.get(path) {
        // } else {
        //     let data = Dataset::open(path)?;
        //     let r = lookup_in_dataset(&data, lng, lat);
        //     // self.cache.insert(path.to_path_buf(), data);

        //     r
        // }
    }
}
