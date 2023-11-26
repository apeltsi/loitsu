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
use std::io::Read;
use crate::info;
use loitsu::Preferences;

pub fn build_assets(out_dir: &PathBuf, force: bool) {
    let asset_path = std::env::current_dir().unwrap().join("assets");
    let shard_dir = out_dir.join("shards");

    let checksum = get_assets_checksum(&asset_path);
    // lets compare the checksum to the previous one
    let mut old_checksum = String::new();
    let mut old_checksum_path = shard_dir.clone();
    old_checksum_path.push("checksum");
    if old_checksum_path.exists() {
        let mut file = File::open(old_checksum_path).unwrap();
        file.read_to_string(&mut old_checksum).unwrap();
    }
    if checksum == old_checksum && !force {
        info!("Assets haven't changed. No shards were generated.");
        return;
    }
    
    info!("Building assets...");

    let files = read_files("assets");
    
    let mut scenes = Vec::new();
    {
        let files = files.clone();
        for file in files {
            if file.name.ends_with(".scene.json") {
                let path = file.path.strip_prefix(asset_path.clone()).unwrap();
                let name = path.to_str().unwrap().replace(".scene.json", "").replace("\\", "/");
                scenes.push((name.to_owned(), String::from_utf8(file.data).unwrap()));
                info!("Found scene: {}", name);
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

    // lets load our Preferences
    let preferences = {
        let mut path = asset_path.clone();
        path.push("preferences.json");
        if path.exists() {
            let mut file = File::open(path).unwrap();
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str::<Preferences>(&data).unwrap()
        } else {
            panic!("Couldn't find preferences.json! Please create one in the assets directory with the required fields.")
        }
    };

    info!("Building {} scenes and {} scripts...", scenes.len(), scripts.len());
    let scenes = loitsu::build_scenes(scenes, scripts);
    info!("Generating shards...");
    let (shards, static_shard) = shard_gen::generate_shards(scenes, script_sources, &preferences);

    let overrides = get_asset_overrides(&asset_path);
    // lets make sure the shard dir exists
    let s_path = Path::new(&shard_dir);

    if !s_path.exists() {
        std::fs::create_dir_all(s_path).unwrap();
    }

    // lets quickly clear the shard directory
    for entry in std::fs::read_dir(s_path).unwrap() {
        let entry = entry.unwrap();
        std::fs::remove_file(entry.path()).unwrap();
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
        file.write_all(&data).unwrap();
    }

    // lets write the static shards
    let data = static_shard.encode();
    total_size += data.len();
    let mut path = shard_dir.clone();
    path.push("static");
    path.set_extension("shard");
    let mut file = File::create(path).unwrap();
    file.write_all(&data).unwrap();

    // Now we'll write the checksum
    let mut path = shard_dir.clone();
    path.push("checksum");
    let mut file = File::create(path).unwrap();
    file.write_all(checksum.as_bytes()).unwrap();
    
    info!("Generated {} shard(s) with a total size of {}", shard_count + 1, format_size(total_size));
}

fn get_assets_checksum(path: &PathBuf) -> String {
    checksumdir::checksumdir(path.to_str().unwrap()).unwrap()
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
        let entry = entry.expect("Couldn't read assets directory! Are you in the correct directory?");
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
    path.push("overrides.json");
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
