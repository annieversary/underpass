pub mod cached;
pub mod gdal;

use std::path::Path;

use thiserror::Error;

use self::{cached::CachedGdalElevationMap, gdal::GdalElevationMap};

#[derive(Error, Debug)]
pub enum ElevationError {
    #[error("io error")]
    Network(#[from] std::io::Error),
    #[error("gdal error")]
    Gdal(#[from] ::gdal::errors::GdalError),
    #[error("coordinate not found")]
    CoordNotFound,
}

pub enum ElevationMap {
    Gdal(GdalElevationMap),
    CachedGdal(CachedGdalElevationMap),
}

impl ElevationMap {
    pub fn new_gdal(path: &Path) -> Result<Self, ElevationError> {
        Ok(Self::Gdal(GdalElevationMap::new(path)?))
    }

    pub fn cached(self) -> Self {
        match self {
            ElevationMap::Gdal(gdal) => Self::CachedGdal(CachedGdalElevationMap::new(gdal)),
            ElevationMap::CachedGdal(_) => self,
        }
    }

    pub fn cached_if(self, condition: bool) -> Self {
        if condition {
            self.cached()
        } else {
            self
        }
    }

    /// looks up the elevation for the given coordinates
    pub async fn lookup(&self, lng: f64, lat: f64) -> Result<i32, ElevationError> {
        match self {
            ElevationMap::Gdal(gdal) => gdal.lookup(lng, lat),
            ElevationMap::CachedGdal(cached) => cached.lookup(lng, lat).await,
        }
    }

    /// looks up the elevation for the given coordinates, or returns 0 if there's an error
    pub async fn lookup_or_0(&self, lng: f64, lat: f64) -> i32 {
        self.lookup(lng, lat).await.unwrap_or(0)
    }
}
