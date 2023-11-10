use std::collections::HashMap;
use std::io::{Write, Read};
use super::AssetError;
use super::asset::{Asset, ImageAsset};

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
    pub fn consume(&mut self) -> Result<ConsumedShard, AssetError> {
        let mut assets = HashMap::new();
        for (name, file) in self.assets.drain() {
            let asset: Box<dyn Asset> = match name.split(".").last().unwrap() {
                "png" => {
                   Box::new(ImageAsset::from_bytes(file.data, &file.name))
                },
                _ => {
                    return Err(AssetError::new("Unknown file type"));
                }
            };
            assets.insert(name, asset);
        }
        Ok(ConsumedShard {
            name: self.name.clone(),
            assets,
            is_initialized: false
        })
    }
}

pub struct ConsumedShard {
    pub name: String,
    pub assets: HashMap<String, Box<dyn Asset>>,
    pub is_initialized: bool
}

impl ConsumedShard {
    pub fn initialize(&mut self, graphics_device: &wgpu::Device, queue: &wgpu::Queue) {
        // assets such as sprites have to be initialized 
        // (with access to the graphics device)
        for (_, asset) in self.assets.iter_mut() {
            asset.initialize(graphics_device, queue).unwrap();
        }
        self.is_initialized = true;
    }
}
