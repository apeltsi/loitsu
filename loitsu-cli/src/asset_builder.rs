use std::path::{PathBuf, Path};
use walkdir::WalkDir;
use loitsu::scripting::ScriptingSource;
use std::str;
use crate::shard_gen;
use std::fs::File;
use std::io::Write;

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
    for file in files {
        if file.name.ends_with(".rn") {
            scripts.push(ScriptingSource {
                name: file.name,
                source: String::from_utf8(file.data).unwrap(),
            });
        }
    }

    println!("Building {} scenes and {} scripts...", scenes.len(), scripts.len());
    let scenes = loitsu::build_scenes(scenes, scripts);
    println!("Generating shards...");
    let shards = shard_gen::generate_shards(scenes);
    let shard_dir = out_dir.join("shards");

    // lets make sure the shard dir exists
    let s_path = Path::new(&shard_dir);

    if !s_path.exists() {
        std::fs::create_dir_all(s_path);
    }
    let mut total_size: usize = 0;
    let shard_count = shards.len();
    for shard in shards {
        let data = shard.encode();
        total_size += data.len();
        let mut path = shard_dir.clone(); 
        path.push(shard.name);
        path.set_extension("shard");
        // now lets write the data
        let mut file = File::create(path).unwrap();
        file.write_all(&data);
    }
    println!("Generated {} shard(s) with a total size of {}", shard_count, format_size(total_size));
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
