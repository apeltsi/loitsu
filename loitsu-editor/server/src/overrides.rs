use std::io::Cursor;
use crate::PathBuf;
use serde_json::Value;
use std::fs;
use std::collections::HashMap;
use image::io::Reader as ImageReader;

#[derive(Clone, Debug)]
pub struct AssetOverride {
    pub resolution_multiplier: Option<f32>,
}

pub fn get_asset_overrides(path: &PathBuf) -> HashMap<String, AssetOverride> {
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

// right now this is pretty much a 1 to 1 copy of the loitsu-cli code
pub fn handle_override(file_path: PathBuf, file_data: Vec<u8>, asset_override: &AssetOverride) -> Vec<u8> {
    let extension = file_path.extension().unwrap().to_str().unwrap();
    match extension {
        "png" | "jpeg" => {
            let data = ImageReader::new(Cursor::new(file_data)).with_guessed_format().unwrap().decode().unwrap();
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
            buffer
        },
        _ => {
            file_data
        }
    }
}
