use std::time::Duration;

use ahash::RandomState;
use geojson::FeatureCollection;
use moka::future::Cache;

use crate::search::{Bbox, GeocodeaArea};

#[derive(Clone)]
pub struct Caches {
    pub overpass: OverpassCache,
}

impl Caches {
    pub fn new() -> Self {
        let overpass = Cache::builder()
            .max_capacity(100)
            .time_to_live(Duration::from_secs(30 * 60))
            .time_to_idle(Duration::from_secs(10 * 60))
            // we use ahash because it's faster for big keys
            .build_with_hasher(ahash::RandomState::default());

        Self { overpass }
    }
}

pub type OverpassCache =
    Cache<(String, Bbox), (FeatureCollection, Vec<GeocodeaArea>, String), RandomState>;
