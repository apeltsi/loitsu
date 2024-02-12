use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{asset::Asset, AssetError};

pub struct AssetReference {
    asset: Arc<Mutex<Asset>>,
    runtime_version: u32, // Incremented if the asset is changed at runtime, (hot-reload etc)
}

impl AssetReference {
    pub fn new(asset: Arc<Mutex<Asset>>) -> AssetReference {
        AssetReference {
            asset,
            runtime_version: 0,
        }
    }

    pub fn get_asset(&self) -> Arc<Mutex<Asset>> {
        self.asset.clone()
    }

    pub fn get_version(&self) -> u32 {
        self.runtime_version
    }

    pub fn increment_version(&mut self, amount: u32) {
        self.runtime_version += amount;
    }

    pub fn update(&mut self, asset: Arc<Mutex<Asset>>) {
        self.asset = asset;
        self.runtime_version += 1;
    }

    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &HashMap<String, Arc<Mutex<AssetReference>>>,
    ) -> Result<(), AssetError> {
        self.asset.lock().unwrap().initialize(device, queue, assets)
    }
}
