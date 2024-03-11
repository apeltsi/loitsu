use crate::info;
use crate::shard_gen;
use loitsu::scripting::ScriptingSource;
use loitsu::Preferences;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str;
use walkdir::WalkDir;

pub async fn build_assets(out_dir: &PathBuf, force: bool, release: bool) {
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
                let name = path
                    .to_str()
                    .unwrap()
                    .replace(".scene.json", "")
                    .replace("\\", "/");
                scenes.push((name.to_owned(), String::from_utf8(file.data).unwrap()));
                info!("Found scene: {}", name);
            }
        }
    }

    let mut scripts = Vec::new();
    for file in files {
        if file.name.ends_with(".rn") {
            scripts.push(ScriptingSource {
                name: file.name.clone(),
                source: String::from_utf8(file.data.clone()).unwrap(),
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

    info!(
        "Building {} scenes and {} scripts...",
        scenes.len(),
        scripts.len()
    );
    let scenes = loitsu::build_scenes(scenes, scripts.clone());
    info!("Generating shards...");
    let (shards, static_shard) = shard_gen::generate_shards(scenes, scripts, &preferences);

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
        let data = shard.encode(release).await;
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

    info!(
        "Generated {} shard(s) with a total size of {}",
        shard_count + 1,
        format_size(total_size)
    );
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
        let entry =
            entry.expect("Couldn't read assets directory! Are you in the correct directory?");
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
