use std::collections::HashMap;
use std::io::Write;

use crate::scene_management::Scene;
use crate::scripting::ScriptingSource;

#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct StaticShard {
    shard_map: HashMap<String, Vec<String>>, // A mapping of scene -> required shards
    scripts: Vec<ScriptingSource>,
    scenes: Vec<Scene>
}



#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct StaticShardFile {
    name: String,
    data: Vec<u8>,
}

impl StaticShard {
    pub fn new(shard_map: HashMap<String, Vec<String>>, scripts: Vec<ScriptingSource>, scenes: Vec<Scene>) -> StaticShard {
        StaticShard {
            shard_map,
            scripts,
            scenes
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
}
