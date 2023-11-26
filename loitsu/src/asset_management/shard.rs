use std::collections::HashMap;
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};

use super::AssetError;
use super::asset::{Asset, image_from_bytes};

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn as spawn_local;

#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct Shard {
    name: String,
    assets: HashMap<String, ShardFile>
}

#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct ShardFile {
    name: String,
    data: Vec<u8>,
}

impl Shard {
    pub fn new(name: String) -> Shard {
        Shard {
            name,
            assets: HashMap::new()
        }
    }

    pub fn add_file(&mut self, name: String, data: Vec<u8>) {
        self.assets.insert(name.clone(), ShardFile {
            name,
            data
        });
    }

    pub fn get_file(&self, name: &str) -> Option<&ShardFile> {
        self.assets.get(name)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn encode(&self) -> Vec<u8> {
        let uncompressed_bytes = bitcode::encode(&self).unwrap();

        // now lets compress it
        let mut encoder = zstd::stream::Encoder::new(Vec::new(), 0).unwrap();
        encoder.write_all(&uncompressed_bytes).unwrap();
        let compressed_bytes = encoder.finish().unwrap();
        compressed_bytes
    }

    pub fn decode(bytes: &[u8]) -> Shard {
        let mut decoder = zstd::stream::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
        let mut uncompressed_bytes = Vec::new();
        decoder.read_to_end(&mut uncompressed_bytes).unwrap();
        let shard = bitcode::decode::<Shard>(&uncompressed_bytes).unwrap();
        shard
    }

    /// Consumes the shard, returning the parsed assets
    pub async fn consume(&mut self) -> Result<ConsumedShard, AssetError> {
        let assets = Arc::new(Mutex::new(HashMap::new()));
        #[cfg(not(target_arch = "wasm32"))]
        let mut futures = Vec::new();
        for (name, file) in self.assets.drain() {
            let assets = assets.clone();
            let task = async move {
                let asset: Arc<Mutex<Asset>> = match name.split(".").last().unwrap() {
                    "png" => {
                        Arc::new(Mutex::new(image_from_bytes(file.data, &file.name)))
                    },
                    _ => {
                        #[cfg(not(target_arch = "wasm32"))]
                        return Err(AssetError::new("Unknown file type"));
                        #[cfg(target_arch = "wasm32")]
                        return;
                    }
                };
                assets.lock().unwrap().insert(name, asset);
                #[cfg(not(target_arch = "wasm32"))]
                Ok(())
            };
            #[cfg(target_arch = "wasm32")]
            task.await;
            #[cfg(not(target_arch = "wasm32"))] {
                let future = spawn_local(task);
                futures.push(future);
            }
        }

        // no multi-threading on wasm :(
        #[cfg(not(target_arch = "wasm32"))]
        for future in futures {
            future.await??;
        }
        Ok(ConsumedShard {
            name: self.name.clone(),
            assets: assets.clone().lock().unwrap().clone(),
            is_initialized: false
        })
    }
}

pub struct ConsumedShard {
    pub name: String,
    pub assets: HashMap<String, Arc<Mutex<Asset>>>,
    pub is_initialized: bool
}

impl ConsumedShard {
    pub fn initialize(&mut self, graphics_device: &wgpu::Device, queue: &wgpu::Queue) {
        // assets such as sprites have to be initialized 
        // (with access to the graphics device)
        for (_, asset) in self.assets.iter_mut() {
            asset.lock().unwrap().initialize(graphics_device, queue).unwrap();
        }
        self.is_initialized = true;
    }

    pub fn get_asset(&self, name: &str) -> Option<&Arc<Mutex<Asset>>> {
        self.assets.get(name)
    }
}
