use std::path::PathBuf;

use underpass::taginfo::{self, taginfo_path};

#[tokio::main]
async fn main() {
    // we only care if the error is a line parse
    if let Err(err @ dotenv::Error::LineParse(..)) = dotenv::dotenv() {
        panic!("{:?}", err);
    }

    let data_path: PathBuf = std::env::var("DATA_PATH")
        .expect("failed to get DATA")
        .into();

    let taginfo_path = taginfo_path(&data_path);

    println!("updating taginfo");
    taginfo::update_taginfo(taginfo_path).await.unwrap();
}
