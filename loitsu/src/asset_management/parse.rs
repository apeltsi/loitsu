use std::sync::{Arc, Mutex};

use crate::asset_management::shard::ShardFileType;

use super::{
    asset::{texture_from_bytes, Asset},
    asset_meta::AssetMeta,
    shard::ShardFile,
    texture_asset::TextureMeta,
    AssetError,
};

pub fn parse(file: ShardFile) -> Result<Arc<Mutex<Asset>>, AssetError> {
    match file.r#type {
        ShardFileType::Texture => Ok(Arc::new(Mutex::new(texture_from_bytes(
            file.data, &file.name,
        )))),
        ShardFileType::FileMeta => {
            let meta: AssetMeta = bitcode::decode(&file.data).unwrap();
            match meta {
                AssetMeta::TextureMeta(meta) => {
                    return Ok(Arc::new(Mutex::new(Asset::TextureMeta(TextureMeta::new(
                        meta.get_target(),
                        meta.get_uv(),
                        meta.get_format(),
                    )))));
                }
                AssetMeta::None => {
                    return Err(AssetError::new("None type meta found. These should not be included in the shard. Did shard-gen provide any errors or warnings?"));
                }
            }
        }
    }
}
