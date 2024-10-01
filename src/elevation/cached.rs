use std::{path::PathBuf, sync::Arc};

use ahash::HashMap;
use gdal::Dataset;
use tokio::sync::Mutex;

use super::{
    gdal::{lookup_in_dataset, GdalElevationMap},
    ElevationError,
};

pub struct CachedGdalElevationMap {
    map: GdalElevationMap,
    cache: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<Dataset>>>>>,
}

impl CachedGdalElevationMap {
    pub(super) fn new(map: GdalElevationMap) -> Self {
        Self {
            map,
            cache: Default::default(),
        }
    }

    /// looks up the elevation for the given coordinates
    pub(super) async fn lookup(&self, lng: f64, lat: f64) -> Result<i32, ElevationError> {
        let path = self.map.dataset_path(lng, lat)?;

        let data = {
            let cache = self.cache.lock().await;
            cache.get(path).cloned()
        };

        if let Some(data) = data {
            let data = data.lock().await;
            lookup_in_dataset(&data, lng, lat)
        } else {
            let data = Dataset::open(path)?;
            let r = lookup_in_dataset(&data, lng, lat);

            {
                let mut cache = self.cache.lock().await;
                cache.insert(path.to_path_buf(), Arc::new(Mutex::new(data)));
            }

            r
        }
    }
}
