use self::{static_shard::StaticShard, asset::Asset};
use crate::log;
use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on as spawn_local;

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
        let pending_tasks = Arc::new(AtomicUsize::new(1));
        let assets = Assets {
            shards: Vec::new(),
            static_shard: None,
        };
        let assets = Arc::new(Mutex::new(assets));
        let assets_clone = assets.clone();
        let pending_tasks_clone = pending_tasks.clone();
        spawn_local(async move {
            let result = get_file::get_file("static.shard".to_string()).await;

            match result {
                Ok(file) => {
                    log!("Successfully loaded static shard");
                    let static_shard = StaticShard::decode(&file);
                    let mut assets = assets_clone.lock().unwrap();
                    assets.static_shard = Some(static_shard);
                    pending_tasks_clone.fetch_sub(1, Ordering::SeqCst);
                },
                Err(e) => {
                    log!("Failed to load static shard: {:?}", e.message);
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
        let pending_tasks = self.pending_tasks.clone();
        spawn_local(async move {
            let mut assets = assets.lock().unwrap();
            
            // now lets fetch the shards, and join the futures
            let mut futures = Vec::new();
            for shard in shards {
                futures.push(get_file::get_file(shard + ".shard"));
                pending_tasks.fetch_add(1, Ordering::SeqCst);
            }
            let mut results = Vec::new();
            for future in futures {
                results.push(future.await);
            }

            // now lets decode the shards
            
            for result in results {
                match result {
                    Ok(file) => {
                        log!("Successfully loaded shard");
                        let mut shard = shard::Shard::decode(&file);
                        let consumed_shard = shard.consume().unwrap();
                        assets.shards.push(consumed_shard);
                        pending_tasks.fetch_sub(1, Ordering::SeqCst);
                    },
                    Err(e) => {
                        log!("Failed to load shard: {:?}", e.message);
                    }
                }
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
        }
    }

    pub fn get_asset(&self, name: &str) -> Option<&Box<Asset>>  {
        let assets = self.assets.lock().unwrap();
        for shard in &assets.shards {
            if let Some(asset) = shard.get_asset(name) {

            }
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
