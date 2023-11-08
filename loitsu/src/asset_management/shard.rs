use std::collections::HashMap;
use std::io::{Write, Read};

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
}
