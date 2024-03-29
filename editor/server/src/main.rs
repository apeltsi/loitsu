use loitsu::asset_management::shard::guess_file_type;
use loitsu::asset_management::AssetMetaPair;
use loitsu::Preferences;
use loitsu_asset_gen::{default_meta, perform_pre_processing};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;
use warp;
use warp::http::Response;
use warp::Filter;

#[tokio::main]
async fn main() {
    let asset_path = std::env::current_dir().unwrap().join("assets");
    let preferences = parse_preferences(asset_path.clone());

    let exe_path = std::env::current_exe().unwrap();
    let path = exe_path.parent().unwrap().join("editor_assets");
    let asset_path_clone = asset_path.clone();
    let cors = warp::cors()
        .allow_origin("http://localhost:5173")
        .allow_methods(vec!["GET", "POST", "DELETE"]);
    let preferences_clone = preferences.clone();
    let mut open_scene_path = preferences_clone.default_scene.clone();
    // lets check if the user provided a scene to open
    for arg in std::env::args() {
        if arg.starts_with("--scene=") {
            open_scene_path = arg.replace("--scene=", "");
        }
    }
    let open_scene_clone = open_scene_path.clone();
    let main_scene_route = warp::get()
        .and(warp::path("LOITSU_MAIN_SCENE"))
        .map(move || {
            let mut path = asset_path_clone.clone();
            path.push(format!("{}.scene.json", open_scene_clone));
            let mut file = File::open(path).unwrap();
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            Response::builder()
                .header(
                    "Cache-Control",
                    "no-store, no-cache, must-revalidate, proxy-revalidate",
                )
                .body(data)
        })
        .with(cors.clone());
    let asset_path_clone = asset_path.clone();
    let scripts_route = warp::get()
        .and(warp::path("LOITSU_ALL_SCRIPTS"))
        .map(move || {
            let files = read_files(asset_path_clone.clone().to_str().unwrap());
            let mut scripts = Vec::new();
            for file in files {
                if file.name.ends_with(".rn") {
                    scripts.push(String::from_utf8(file.data.clone()).unwrap());
                }
            }
            Response::builder()
                .header(
                    "Cache-Control",
                    "no-store, no-cache, must-revalidate, proxy-revalidate",
                )
                .body(serde_json::to_string(&scripts).unwrap())
        })
        .with(cors.clone());
    let asset_path_clone = asset_path.clone();
    let assets_route = warp::get()
        .and(warp::path("assets"))
        .and(warp::path::tail())
        .and_then(move |tail: warp::path::Tail| {
            use tokio::fs::File;
            use tokio::io::AsyncReadExt;
            let asset_path = asset_path_clone.clone();
            async move {
                let mut path = asset_path;
                path.push(tail.as_str());

                if !path.exists() {
                    return Ok::<_, warp::Rejection>(Response::builder().status(404).body(vec![]));
                }
                let mut file = File::open(&path).await.unwrap();
                let mut data = Vec::new();
                file.read_to_end(&mut data).await.unwrap();
                // now lets check if a .meta file exists for this asset
                let mut meta_path = path.clone();
                meta_path.set_extension("meta");
                let file_type = guess_file_type(&path.to_str().unwrap());
                let mut meta = default_meta(&file_type, tail.as_str());
                if meta_path.exists() {
                    let mut file = File::open(meta_path).await.unwrap();
                    // deserialize the json
                    let mut data = Vec::new();
                    file.read_to_end(&mut data).await.unwrap();
                    meta = serde_json::from_slice(&data).unwrap();
                }
                let asset_meta = AssetMetaPair {
                    asset: perform_pre_processing(&path, &meta).await,
                    meta,
                };

                let data = bitcode::encode(&asset_meta).unwrap();
                Ok::<_, warp::Rejection>(
                    Response::builder()
                        .header(
                            "Cache-Control",
                            "no-store, no-cache, must-revalidate, proxy-revalidate",
                        )
                        .body(data),
                )
            }
        })
        .with(cors.clone());
    let route = warp::get().and(
        warp::fs::dir(path.clone())
            .or(main_scene_route)
            .or(scripts_route)
            .or(assets_route),
    );
    // the client will post a json object with the scene data,
    // this will be save to the default scene
    let save_route = warp::post()
        .and(warp::path("save_scene"))
        .and(warp::body::bytes())
        .and_then(move |body: warp::hyper::body::Bytes| {
            let asset_path = asset_path.clone();
            let mut path = asset_path.clone();
            path.push(format!("{}.scene.json", open_scene_path));
            let mut file = File::create(path).unwrap();
            use std::io::Write;
            file.write_all(&body).unwrap();
            async move { Ok::<_, warp::Rejection>(Response::builder().body("")) }
        });
    let route = route.or(save_route);
    println!("Editor live at http://localhost:5969");
    warp::serve(route).run(([127, 0, 0, 1], 5969)).await;
}

fn parse_preferences(asset_path: PathBuf) -> Preferences {
    {
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
    }
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AssetFile {
    path: PathBuf,
    name: String,
    extension: String,
    data: Vec<u8>,
}
