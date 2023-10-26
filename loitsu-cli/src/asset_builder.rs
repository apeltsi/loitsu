use std::path::PathBuf;
use walkdir::WalkDir;
use loitsu::scripting::ScriptingSource;
use std::str;

pub fn build_assets(_out_dir: &PathBuf) {
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
    for scene in scenes {
        println!("{}", scene.to_json());
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
