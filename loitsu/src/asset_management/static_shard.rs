use std::collections::HashMap;
use std::io::Write;
use std::io::Read;

use crate::Preferences;
use crate::scene_management::Scene;
use crate::scripting::ScriptingSource;

#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct StaticShard {
    shard_map: HashMap<String, Vec<String>>, // A mapping of scene -> required shards
    scripts: Vec<ScriptingSource>,
    scenes: Vec<Scene>,
    preferences: Preferences
}



#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct StaticShardFile {
    name: String,
    data: Vec<u8>,
}

impl StaticShard {
    pub fn new(shard_map: HashMap<String, Vec<String>>, scripts: Vec<ScriptingSource>, scenes: Vec<Scene>, preferences: Preferences) -> StaticShard {
        StaticShard {
            shard_map,
            scripts,
            scenes,
            preferences
        }
    }
    
    pub fn encode(&self) -> Vec<u8> {
        let uncompressed_bytes = bitcode::encode(&self).unwrap();

        // now lets compress it
        let mut encoder = zstd::stream::Encoder::new(Vec::new(), 0).unwrap();
        encoder.write_all(&uncompressed_bytes).unwrap();
        let compressed_bytes = encoder.finish().unwrap();
        compressed_bytes
    }

    pub fn decode(data: &[u8]) -> StaticShard {
        let mut decoder = zstd::stream::Decoder::new(data).unwrap();
        let mut uncompressed_bytes = Vec::new();
        decoder.read_to_end(&mut uncompressed_bytes).unwrap();
        let shard: StaticShard = bitcode::decode(&uncompressed_bytes).unwrap();
        shard
    }
}
