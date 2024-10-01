use std::path::PathBuf;

use underpass::taginfo::{self, taginfo_path};

#[tokio::main]
async fn main() {
    let data_path: PathBuf = std::env::var("DATA_PATH")
        .expect("failed to get DATA")
        .into();

    let taginfo_path = taginfo_path(&data_path);

    println!("updating taginfo");
    taginfo::update_taginfo(taginfo_path).await.unwrap();
}
