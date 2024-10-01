use std::path::{Path, PathBuf};

/// Returns the path of taginfo.json, based on the path of the data folder
pub fn taginfo_path(data_path: &Path) -> PathBuf {
    let mut taginfo_path = data_path.to_path_buf();
    taginfo_path.push("taginfo");
    taginfo_path.push("taginfo.json");
    taginfo_path
}
