use self::{static_shard::StaticShard, asset::Asset};
use crate::{log_asset as log, error};
use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn as spawn_local;

use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};

pub mod shard;
pub mod static_shard;
pub mod get_file;
pub mod asset;
pub mod image_asset;

lazy_static!{
    pub static ref ASSET_MANAGER: Arc<Mutex<AssetManager>> = Arc::new(Mutex::new(AssetManager::new()));
}

pub struct AssetManager {
    pub pending_tasks: Arc<AtomicUsize>,
    pub assets: Arc<Mutex<Assets>>
}

pub struct Assets {
    pub shards: Vec<shard::ConsumedShard>,
    pub static_shard: Option<static_shard::StaticShard>,
}

impl AssetManager {
    pub fn new() -> AssetManager {
        #[cfg(not(feature = "direct_asset_management"))]
        let pending_tasks = Arc::new(AtomicUsize::new(1));
        // If direct asset management is not in use, we'll have to wait for the static shard
        #[cfg(feature = "direct_asset_management")]
        let pending_tasks = Arc::new(AtomicUsize::new(0));
        let assets = Assets {
            shards: Vec::new(),
            static_shard: None,
        };
        let assets = Arc::new(Mutex::new(assets));
        let assets_clone = assets.clone();
        let pending_tasks_clone = pending_tasks.clone();
        #[cfg(not(feature = "direct_asset_management"))]
        spawn_local(async move {
            let result = get_file::get_file("shards/static.shard".to_string()).await;

            match result {
                Ok(file) => {
                    log!("Successfully loaded static shard");
                    let static_shard = StaticShard::decode(&file);
                    let mut assets = assets_clone.lock().unwrap();
                    assets.static_shard = Some(static_shard);
                    pending_tasks_clone.fetch_sub(1, Ordering::SeqCst);
                },
                Err(e) => {
                    error!("Failed to load static shard: {:?}", e.message);
                    // TODO: On web platforms this could show some error to the user
                }
            }
        });
        AssetManager {
            pending_tasks,
            assets,
        }
    }

    pub fn request_shards(&mut self, shards: Vec<String>) {
        let assets = self.assets.clone();
        self.pending_tasks.fetch_add(shards.len(), Ordering::SeqCst);
        let pending_tasks = self.pending_tasks.clone();
        spawn_local(async move {
            // now lets fetch the shards, and join the futures
            let mut futures = Vec::new();
            for shard in shards {
                futures.push(get_file::get_file(format!("shards/{}", shard + ".shard")));
            }
            let mut results = Vec::new();
            for future in futures {
                let value = future.await;
                results.push(value);
            }

            // now lets decode the shards
            for result in results.drain(..) {
                let pending_tasks = pending_tasks.clone();
                let assets = assets.clone();
                spawn_local(async move {
                    match result {
                        Ok(file) => {
                            let mut shard = shard::Shard::decode(&file);
                            let consumed_shard = shard.consume().await.unwrap();
                            let mut assets = assets.lock().unwrap();
                            assets.shards.push(consumed_shard);
                            pending_tasks.fetch_sub(1, Ordering::SeqCst);
                            log!("Successfully loaded shard: '{}'", shard.get_name());
                        },
                        Err(e) => {
                            error!("Failed to load shard: {:?}", e.message);
                        }
                    }
                });
            }

        });
    }

    pub fn initialize_shards(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.pending_tasks.load(Ordering::SeqCst) > 0 {
            return;
        }
        let mut assets = self.assets.lock().unwrap();
        for shard in &mut assets.shards {
            if shard.is_initialized {
                continue;
            }
            shard.initialize(device, queue);
            log!("Shard '{}' initialized", shard.name);
        }
    }

    pub fn get_asset(&self, name: &str) -> Option<Arc<Mutex<Asset>>>  {
        #[cfg(not(feature = "direct_asset_management"))]
        {
            let assets = self.assets.lock().unwrap();
            for shard in &assets.shards {
                if let Some(asset) = shard.get_asset(name) {
                    return Some(asset.clone());
                }
            }
        }
        #[cfg(target_arch = "wasm32")] // currently we only support direct asset management on web
        #[cfg(feature = "direct_asset_management")]
        {
            // The asset wasn't in a shard, and we have direct asset management enabled
            // so we'll try to load it from from the local asset server
            log!("Asset not found in shards, trying to load from local asset server");
        }
        None
    }
}

#[derive(Debug)]
pub struct AssetError {
    message: String,
}

impl AssetError {
    pub fn new(message: &str) -> AssetError {
        AssetError {
            message: message.to_owned(),
        }
    }
}

impl From<std::io::Error> for AssetError {
    fn from(value: std::io::Error) -> Self {
        AssetError::new(&format!("{:?}", value))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<tokio::task::JoinError> for AssetError {
    fn from(value: tokio::task::JoinError) -> Self {
        AssetError::new(&format!("{:?}", value))
    }
}
