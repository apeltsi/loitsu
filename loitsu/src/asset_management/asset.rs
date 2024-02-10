use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use super::texture_asset::{TextureAsset, TextureMeta};
use super::{asset_reference::AssetReference, AssetError};
pub enum Asset {
    None, // used temporarily when loading assets
    Texture(TextureAsset),
    TextureMeta(TextureMeta),
}

impl Asset {
    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &HashMap<String, Arc<Mutex<AssetReference>>>,
    ) -> Result<(), AssetError> {
        match self {
            Asset::Texture(image) => image.initialize(device, queue),
            Asset::TextureMeta(meta) => meta.initialize(assets),
            _ => Ok(()),
        }
    }
}

pub fn texture_from_bytes(bytes: Vec<u8>, name: &str) -> Asset {
    Asset::Texture(TextureAsset::from_bytes(bytes, name))
}
