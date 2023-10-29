use std::path::{PathBuf, Path};
use walkdir::WalkDir;
use loitsu::scripting::ScriptingSource;
use std::str;
use crate::shard_gen;
use std::fs::File;
use std::io::Write;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

pub fn build_assets(out_dir: &PathBuf) {
    let files = read_files("assets");
    
    let mut scenes = Vec::new();
    {
        let files = files.clone();
        for file in files {
            if file.name.ends_with(".scene.json") {
                let name = file.name.split(".").collect::<Vec<&str>>()[0];
                scenes.push((name.to_owned(), String::from_utf8(file.data).unwrap()));
            }
        }
    }

    let mut scripts = Vec::new();
    let mut script_sources = Vec::new();
    for file in files {
        if file.name.ends_with(".rn") {
            scripts.push(ScriptingSource {
                name: file.name.clone(),
                source: String::from_utf8(file.data.clone()).unwrap(),
            });
            script_sources.push(ScriptingSource {
                name: file.name,
                source: String::from_utf8(file.data).unwrap(),
            });
        }
    }

    println!("Building {} scenes and {} scripts...", scenes.len(), scripts.len());
    let scenes = loitsu::build_scenes(scenes, scripts);
    println!("Generating shards...");
    let (shards, static_shard) = shard_gen::generate_shards(scenes, script_sources);
    let shard_dir = out_dir.join("shards");
    let asset_path = std::env::current_dir().unwrap().join("assets");
    let overrides = get_asset_overrides(&asset_path);
    // lets make sure the shard dir exists
    let s_path = Path::new(&shard_dir);

    if !s_path.exists() {
        std::fs::create_dir_all(s_path);
    }
    let mut total_size: usize = 0;
    let shard_count = shards.len();
    for shard in shards {
        let data = shard.encode(&overrides);
        total_size += data.len();
        let mut path = shard_dir.clone(); 
        path.push(shard.name);
        path.set_extension("shard");
        // now lets write the data
        let mut file = File::create(path).unwrap();
        file.write_all(&data);
    }

    // lets write the static shards
    let data = static_shard.encode();
    total_size += data.len();
    let mut path = shard_dir.clone();
    path.push("static");
    path.set_extension("shard");
    let mut file = File::create(path).unwrap();
    file.write_all(&data);

    println!("Generated {} shard(s) with a total size of {}", shard_count + 1, format_size(total_size));
}

fn format_size(size: usize) -> String {
    if size > 1024 * 1024 * 1024 {
        format!("{:.2} GB", size as f64 / 1024.0 / 1024.0 / 1024.0)
    } else if size > 1024 * 1024 {
        format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
    } else if size > 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AssetFile {
    path: PathBuf,
    name: String,
    extension: String,
    data: Vec<u8>,
}

fn read_files(directory: &str) -> Vec<AssetFile> {
    let mut files = Vec::new();
    let mut path = std::env::current_dir().unwrap();
    path.push(directory);

    // lets recursively walk the directory and read all the files, a bit heavy memory-wise but this
    // is a build step so it should be fine :D
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let extension = path.extension().unwrap().to_str().unwrap().to_string();
            let data = std::fs::read(path).unwrap();
            files.push(AssetFile {
                path: path.to_path_buf(),
                name,
                extension,
                data,
            });
        }
    }
    files
}

pub struct AssetOverride {
    pub resolution_multiplier: Option<f32>,
}

fn get_asset_overrides(path: &PathBuf) -> HashMap<String, AssetOverride> {
    let mut override_map = HashMap::new();
    let mut path = path.clone();
    path.push("assetprefs.json");
    // lets check if the file exists
    if !path.exists() {
        return override_map;
    }

    let data = fs::read_to_string(path).unwrap();
    let v: Value = serde_json::from_str(data.as_str()).unwrap();
    if let Value::Object(map) = v {
        for (key, value) in map {
            if let Value::Object(map) = value {
                let mut asset_override = AssetOverride {
                    resolution_multiplier: None,
                };
                for (key, value) in map {
                    if key == "resolution_multiplier" {
                        if let Value::Number(num) = value {
                            asset_override.resolution_multiplier = Some(num.as_f64().unwrap() as f32);
                        }
                    }
                }
                override_map.insert(key, asset_override);
            }
        }
    }
    override_map
}
