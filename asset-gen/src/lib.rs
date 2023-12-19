use std::{path::PathBuf, io::Cursor, collections::HashMap, io::{Read, Write}, hash::Hasher};
use image::io::Reader as ImageReader;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct AssetOverride {
    pub resolution_multiplier: Option<f32>,
}

/// Returns a unique hash for the asset, its data and overrides
pub fn get_asset_unique_hash(file_path: &PathBuf, file_data: &Vec<u8>, asset_override: &AssetOverride) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(file_path.to_str().unwrap().as_bytes());
    hasher.write(file_data);
    if let Some(resolution_multiplier) = asset_override.resolution_multiplier {
        hasher.write(&resolution_multiplier.to_ne_bytes());
    }
    format!("{:X}", hasher.finish())
}

pub fn get_cached_asset(file_path: &PathBuf, file_data: &Vec<u8>, asset_override: &AssetOverride) -> Option<Vec<u8>> {
    let hash = get_asset_unique_hash(file_path, file_data, asset_override);
    let mut path = std::env::current_dir().unwrap();
    path.push(".loitsu");
    path.push("asset_cache");
    path.push(hash);
    if path.exists() {
        let mut file = std::fs::File::open(path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        Some(data)
    } else {
        None
    }
}

fn write_cached_asset(file_path: &PathBuf, file_data: &Vec<u8>, asset_override: &AssetOverride, data: &Vec<u8>) {
    let hash = get_asset_unique_hash(file_path, file_data, asset_override);
    let mut path = std::env::current_dir().unwrap();
    path.push(".loitsu");
    path.push("asset_cache");
    path.push(hash);
    if !path.exists() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    }
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(data).unwrap();
}

pub async fn handle_override(file_path: PathBuf, file_data: Vec<u8>, asset_override: &AssetOverride) -> Vec<u8> {
    let extension = file_path.extension().unwrap().to_str().unwrap();
    if let Some(cached_data) = get_cached_asset(&file_path, &file_data, asset_override) {
        return cached_data;
    }
    match extension {
        "png" | "jpeg" => {
            let data = ImageReader::new(Cursor::new(file_data.clone())).with_guessed_format().unwrap().decode().unwrap();
            let mut data = data.to_rgba8();
           
            // lets apply the overrides

            // first, resolution_mutliplier
            if let Some(resolution_multiplier) = asset_override.resolution_multiplier {
                let (width, height) = data.dimensions();
                let new_width = (width as f32 * resolution_multiplier).round() as u32;
                let new_height = (height as f32 * resolution_multiplier).round() as u32;
                data = image::imageops::resize(&data, new_width, new_height, image::imageops::FilterType::Nearest);
            }

            // finally lets re-encode the image and return the data
            let mut buffer: Vec<u8> = Vec::new();
            let format = match extension {
                "png" => image::ImageOutputFormat::Png,
                "jpeg" => image::ImageOutputFormat::Jpeg(90),
                _ => image::ImageOutputFormat::Png
            };
            data.write_to(&mut Cursor::new(&mut buffer), format).unwrap();
            write_cached_asset(&file_path, &file_data, asset_override, &buffer);
            buffer
        },
        _ => {
            file_data
        }
    }
}

pub fn get_asset_overrides(path: &PathBuf) -> HashMap<String, AssetOverride> {
    let mut override_map = HashMap::new();
    let mut path = path.clone();
    path.push("overrides.json");
    // lets check if the file exists
    if !path.exists() {
        return override_map;
    }

    let data = std::fs::read_to_string(path).unwrap();
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
