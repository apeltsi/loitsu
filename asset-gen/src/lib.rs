use image::io::Reader as ImageReader;
use loitsu::asset_management::{
    asset_meta::{AssetMeta, TextureMetadata},
    shard::{Shard, ShardFileType},
};
use std::{
    hash::Hasher,
    io::Cursor,
    io::{Read, Write},
    path::PathBuf,
};

/// Returns a unique hash for the asset, its data and overrides
pub fn get_asset_unique_hash(
    file_path: &PathBuf,
    file_data: &Vec<u8>,
    asset_override: &AssetMeta,
) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(file_path.to_str().unwrap().as_bytes());
    hasher.write(file_data);
    hasher.write(bitcode::encode(asset_override).unwrap().as_slice());
    format!("{:X}", hasher.finish())
}

pub fn get_cached_asset(
    file_path: &PathBuf,
    file_data: &Vec<u8>,
    asset_meta: &AssetMeta,
) -> Option<Vec<u8>> {
    let hash = get_asset_unique_hash(file_path, file_data, asset_meta);
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

fn write_cached_asset(
    file_path: &PathBuf,
    file_data: &Vec<u8>,
    asset_meta: &AssetMeta,
    data: &Vec<u8>,
) {
    let hash = get_asset_unique_hash(file_path, file_data, asset_meta);
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

pub async fn resolve_asset(
    shard: &mut Shard,
    asset_relative: &str,
    asset_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_ext = asset_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_lowercase();
    let file_type = match file_ext.as_str() {
        "png" => ShardFileType::Texture,
        "jpeg" => ShardFileType::Texture,
        "meta" => {
            println!("Warning: Meta files should not be references directly. Please reference the meta target instead.");
            return Ok(());
        }
        _ => panic!("Unsupported file type: {}", file_ext),
    };
    let file_meta_path = asset_path.with_extension("meta");
    let file_meta = if !file_meta_path.exists() {
        // get default meta for the file type
        default_meta(&file_type, asset_relative)
    } else {
        let file_meta_data = std::fs::read_to_string(file_meta_path)?;
        let file_meta: AssetMeta = serde_json::from_str(file_meta_data.as_str())?;
        file_meta
    };
    if file_meta != AssetMeta::None {
        shard.add_file(
            asset_relative.to_string(),
            bitcode::encode(&file_meta).unwrap(),
            ShardFileType::FileMeta,
        );
        // now add the file
        let file_data = perform_pre_processing(asset_path, &file_meta).await;
        shard.add_file(format!("{}.TARGET", asset_relative), file_data, file_type);
    } else {
        let file_data = tokio::fs::read(asset_path).await?;
        shard.add_file(asset_relative.to_string(), file_data, file_type);
    }

    Ok(())
}

pub fn default_meta(file_type: &ShardFileType, asset_path: &str) -> AssetMeta {
    match file_type {
        ShardFileType::Texture => {
            let tex_meta = TextureMetadata {
                resolution_multiplier: Some(1.0),
                include_alpha: Some(false),
                uv: Some((0.0, 0.0, 1.0, 1.0)),
                target: format!("{}.TARGET", asset_path),
            };
            AssetMeta::TextureMeta(tex_meta)
        }
        _ => {
            println!("Unsupported file type: {:?}", file_type);
            AssetMeta::None
        }
    }
}

pub async fn perform_pre_processing(file_path: &PathBuf, asset_meta: &AssetMeta) -> Vec<u8> {
    let file_data = tokio::fs::read(file_path).await.unwrap();
    if let Some(cached_data) = get_cached_asset(file_path, &file_data, asset_meta) {
        return cached_data;
    }
    match asset_meta {
        AssetMeta::None => file_data.clone(),
        AssetMeta::TextureMeta(tex_meta) => {
            let img = ImageReader::new(Cursor::new(&file_data))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            // resize if needed
            let img = img.resize(
                (img.width() as f32 * tex_meta.get_resolution_multiplier()) as u32,
                (img.height() as f32 * tex_meta.get_resolution_multiplier()) as u32,
                image::imageops::FilterType::Lanczos3,
            );
            let mut buffer: Vec<u8> = Vec::new();
            img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
                .unwrap();
            write_cached_asset(file_path, &file_data, asset_meta, &buffer);
            buffer
        }
    }
}
