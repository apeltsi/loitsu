pub mod asset;
pub mod asset_reference;
pub mod get_file;
pub mod image_asset;
pub mod parse;
pub mod shard;
pub mod static_shard;

#[allow(unused_imports)]
use self::{asset::Asset, asset_reference::AssetReference, static_shard::StaticShard};
use crate::{error, log_asset as log};
use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn as spawn_local;

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

lazy_static! {
    pub static ref ASSET_MANAGER: Arc<Mutex<AssetManager>> =
        Arc::new(Mutex::new(AssetManager::new()));
}

pub struct AssetManager {
    pub pending_tasks: Arc<AtomicUsize>,
    pub assets: Arc<Mutex<Assets>>,
}

pub struct Assets {
    pub shards: Vec<shard::ConsumedShard>,
    pub assets: HashMap<String, Arc<Mutex<AssetReference>>>,
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
            assets: HashMap::new(),
            static_shard: None,
        };
        let assets = Arc::new(Mutex::new(assets));

        #[cfg(not(feature = "direct_asset_management"))]
        {
            let assets_clone = assets.clone();
            let pending_tasks_clone = pending_tasks.clone();
            spawn_local(async move {
                let result = get_file::get_file("shards/static.shard".to_string()).await;

                match result {
                    Ok(file) => {
                        log!("Successfully loaded static shard");
                        let static_shard = StaticShard::decode(&file);
                        let mut assets = assets_clone.lock().unwrap();
                        assets.static_shard = Some(static_shard);
                        pending_tasks_clone.fetch_sub(1, Ordering::SeqCst);
                    }
                    Err(e) => {
                        error!("Failed to load static shard: {:?}", e.message);
                        // TODO: On web platforms this could show some error to the user
                    }
                }
            });
        }
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

                            for (name, asset) in &consumed_shard.assets {
                                let asset_ref =
                                    Arc::new(Mutex::new(AssetReference::new(asset.clone())));
                                if let Some(old) =
                                    assets.assets.insert(name.to_string(), asset_ref.clone())
                                {
                                    asset_ref
                                        .lock()
                                        .unwrap()
                                        .increment_version(old.lock().unwrap().get_version());
                                }
                            }
                            assets.shards.push(consumed_shard);

                            pending_tasks.fetch_sub(1, Ordering::SeqCst);
                        }
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

    pub fn get_asset(&self, name: &str) -> Arc<Mutex<AssetReference>> {
        #[cfg(not(feature = "direct_asset_management"))]
        {
            let assets = self.assets.lock().unwrap();
            if let Some(asset) = assets.assets.get(name) {
                return asset.clone();
            }
        }
        #[cfg(target_arch = "wasm32")] // currently we only support direct asset management on web
        #[cfg(feature = "direct_asset_management")]
        {
            // first lets check if we already have a asset ref to the requested asset
            if let Some(asset) = self.assets.lock().unwrap().assets.get(name) {
                // the asset is in our local cache, (either being fetched currently or already loaded)
                return asset.clone();
            }
            // The asset wasn't in a shard, and we have direct asset management enabled
            // so we'll try to load it from from the local asset server
            log!("Asset not found in shards, trying to load from local asset server");

            let asset_ref = Arc::new(Mutex::new(AssetReference::new(Arc::new(Mutex::new(
                Asset::None,
            )))));
            self.assets
                .lock()
                .unwrap()
                .assets
                .insert(name.to_owned(), asset_ref.clone());
            let asset_ref_clone = asset_ref.clone();
            let name = name.to_owned();
            #[cfg(feature = "editor")]
            crate::web::add_editor_loading_task("Loading assets");
            spawn_local(async move {
                let result = get_file::get_file(format!("assets/{}", name)).await;
                match result {
                    Ok(file) => {
                        if file.len() == 0 {
                            error!("Failed to load asset: Asset was empty");
                            #[cfg(feature = "editor")]
                            crate::web::remove_editor_loading_task("Loading assets");
                            return;
                        }
                        // lets parse the asset
                        let shard_file = shard::ShardFile { name, data: file };
                        let asset = parse::parse(shard_file).unwrap();
                        asset_ref_clone.lock().unwrap().update(asset);
                    }
                    Err(e) => {
                        error!("Failed to load asset: {:?}", e.message);
                    }
                }
                #[cfg(feature = "editor")]
                crate::web::remove_editor_loading_task("Loading assets");
            });
            return asset_ref;
        }
        // since we guarantee that a asset reference is alway returned
        // we'll have to return a None asset reference for now
        // this reference might be updated later
        #[allow(unreachable_code)]
        {
            let asset = Arc::new(Mutex::new(AssetReference::new(Arc::new(Mutex::new(
                Asset::None,
            )))));
            self.assets
                .lock()
                .unwrap()
                .assets
                .insert(name.to_owned(), asset.clone());
            return asset;
        }
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
