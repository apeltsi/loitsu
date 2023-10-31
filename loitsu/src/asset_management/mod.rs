use self::static_shard::StaticShard;
use crate::log;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on as spawn_local;

use std::sync::{Arc, Mutex};

pub mod shard;
pub mod static_shard;
pub mod get_file;

#[derive(Debug, Clone, PartialEq)]
pub enum AssetManagerStatus {
    Loading,
    Done
}

pub struct AssetManager {
    pub status: Arc<Mutex<AssetManagerStatus>>,
    pub assets: Arc<Mutex<Assets>>
}

pub struct Assets {
    pub shards: Vec<shard::Shard>,
    pub static_shard: Option<static_shard::StaticShard>,
}

impl AssetManager {
    pub fn new() -> AssetManager {
        let status = Arc::new(Mutex::new(AssetManagerStatus::Loading));
        let assets = Assets {
            shards: Vec::new(),
            static_shard: None,
        };
        let assets = Arc::new(Mutex::new(assets));
        let assets_clone = assets.clone();
        let status_clone = status.clone();
        spawn_local(async move {
            let result = get_file::get_file("static.shard").await;
            match result {
                Ok(file) => {
                    log!("Successfully loaded static shard");
                    let static_shard = StaticShard::decode(&file);
                    let mut assets = assets_clone.lock().unwrap();
                    assets.static_shard = Some(static_shard);
                    *status_clone.lock().unwrap() = AssetManagerStatus::Done;
                },
                Err(e) => {
                    log!("Failed to load static shard: {:?}", e.message);
                    // TODO: On web platforms this could show some error to the user
                }
            }
        });
        AssetManager {
            status,
            assets,
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
