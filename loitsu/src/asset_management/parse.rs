use std::sync::{Arc, Mutex};

use super::{shard::ShardFile, asset::{Asset, image_from_bytes}, AssetError};

pub fn parse(file: ShardFile) -> Result<Arc<Mutex<Asset>>, AssetError> {
    match file.name.split(".").last().unwrap() {
        "png" => {
            Ok(Arc::new(Mutex::new(image_from_bytes(file.data, &file.name))))
        },
        _ => {
            return Err(AssetError::new("Unknown file type"));
        }
    }
}
