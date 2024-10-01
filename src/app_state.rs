use std::path::PathBuf;

use crate::{cache::Caches, elevation::ElevationMap};

pub struct AppState {
    pub elevation_map: ElevationMap,
    pub data_path: PathBuf,
    pub caches: Caches,
}

impl AppState {
    pub fn new(data_path: PathBuf, elevation_map: ElevationMap) -> Self {
        AppState {
            elevation_map,
            data_path,
            caches: Caches::new(),
        }
    }
}
