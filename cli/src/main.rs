use cargo_toml::{Dependency, Manifest};
use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use warp::Filter;
mod asset_builder;
mod shard_gen;

#[derive(Debug, Parser)]
#[command(name = "loitsu")]
#[command(about = "Tools useful for development with the Loitsu engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Build {
        #[arg(short, long, default_value = "")]
        target: String,
        #[arg(short, long, default_value = "false")]
        release: bool,
        #[arg(short, long, default_value = "false")]
        force: bool,
    },
    Run {
        #[arg(short, long, default_value = "")]
        target: String,
        #[arg(short, long, default_value = "false")]
        release: bool,
        #[arg(short, long, default_value = "false")]
        force: bool,
    },
    Edit {
        #[arg(short, long, default_value = "")]
        scene: String,
    },
    Clean,
}
#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Build {
            target,
            release,
            force,
        } => build(&target, release, false, force).await,
        Commands::Run {
            target,
            release,
            force,
        } => build(&target, release, true, force).await,
        Commands::Edit { scene } => run_editor(scene),
        Commands::Clean => clean(),
    }
}

fn run_editor(scene: String) {
    let mut command = std::process::Command::new("loitsu-editor");
    if scene != "" {
        command.arg(format!("--scene={}", scene));
    }
    // now lets run the command and wait for it to finish
    let mut child = command.spawn().unwrap();
    child.wait().unwrap();
}

fn clean() {
    let mut path = std::env::current_dir().unwrap();
    path.push(".loitsu");
    path.push("asset_cache");
    if path.exists() {
        fs::remove_dir_all(path.clone()).unwrap();
        println!("loitsu cache has been cleared!");
    } else {
        println!("loitsu cache is already clean!");
    }
}

async fn build(target: &str, release: bool, run: bool, force: bool) {
    if target == "web" {
        info!("Building for web");
        // Now we can build the target
        build_with_args(
            vec!["--target=wasm32-unknown-unknown".to_string()],
            release,
            false,
        );
        let mut out_path = std::env::current_dir().unwrap();
        out_path.push("target");
        out_path.push("wasm32-unknown-unknown");
        if release {
            out_path.push("release");
        } else {
            out_path.push("debug");
        }
        // Lets read the name of the app we just built, this can be determined from the Cargo.toml file
        let mut path = std::env::current_dir().unwrap();
        path.push("Cargo.toml");
        let manifest = Manifest::from_path(path.clone()).unwrap();
        let package_name = manifest.package.unwrap().name;

        let loitsu_version = match &manifest.dependencies["loitsu"] {
            Dependency::Simple(ver) => ver.to_owned(),
            Dependency::Inherited(_inherited) => "Unknown (CUSTOM/DEV)".to_string(),
            Dependency::Detailed(detail) => {
                let ver = &detail.version;
                match ver {
                    Some(version) => version.to_owned(),
                    None => {
                        if let Some(path) = &detail.path {
                            let path = Path::new(path);
                            let mut path = path.to_path_buf();
                            path.push("Cargo.toml");
                            let manifest = Manifest::from_path(path.clone()).unwrap();
                            let package_version = manifest.package.unwrap().version;
                            match package_version {
                                cargo_toml::Inheritable::Set(version) => {
                                    format!("{}dev", version)
                                }
                                cargo_toml::Inheritable::Inherited { .. } => {
                                    "Unknown (CUSTOM/DEV)".to_string()
                                }
                            }
                        } else {
                            "Unknown (CUSTOM/DEV)".to_string()
                        }
                    }
                }
            }
        };
        info!("Running wasm-bindgen...");
        wasm_bindgen(&out_path, package_name.as_str());
        info!("Creating player...");
        // Lets copy the web player files
        generate_player_files(&out_path, &package_name, &loitsu_version, release);

        asset_builder::build_assets(&out_path.join("out"), force, release).await;
        if run {
            start_webserver(&out_path).await;
        } else {
            done("Build Done!");
        }
    } else if target == "" {
        // Now we can build the target
        info!("Building for native");
        let mut out_path = std::env::current_dir().unwrap();
        out_path.push("target");
        if release {
            out_path.push("release");
        } else {
            out_path.push("debug");
        }
        asset_builder::build_assets(&out_path, force, release).await;
        info!("Building native target...");
        build_with_args(vec![], release, run);
    } else {
        panic!("Unsupported target: {}", target);
    }
}

fn generate_player_files(path: &PathBuf, app_name: &str, loitsu_version: &str, is_release: bool) {
    // First lets load the player html file located in /player
    let build_path = path.join("out");
    let raw_player_html = include_str!("../player/index.html");
    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let release_value = if is_release { "true" } else { "false" };
    let player_html = raw_player_html
        .replace("{APP_NAME}", &app_name)
        .replace("{LOITSU_VERSION}", &loitsu_version)
        .replace("{BUILD_DATE}", &build_date)
        .replace("{IS_RELEASE}", release_value);

    // Now lets write the html file to the output directory
    let out_str = build_path.to_str().unwrap();
    let out_dir = Path::new(&out_str);
    let dest_path = &out_dir.join("index.html");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(player_html.as_bytes()).unwrap();

    // Finally lets copy the loitsu logo & js to the output directory
    let logo_bytes = include_bytes!("../player/loitsu.png");
    let dest_path = Path::new(&out_dir).join("loitsu.png");
    let loitsu_js = include_str!("../player/loitsu-web.js")
        .replace("{APP_NAME}", &app_name)
        .replace("{LOITSU_VERSION}", &loitsu_version)
        .replace("{BUILD_DATE}", &build_date)
        .replace("{IS_RELEASE}", release_value);
    let dest_path_js = Path::new(&out_dir).join("loitsu-web.js");

    fs::write(&dest_path, logo_bytes).expect("Unable to write file");
    fs::write(&dest_path_js, loitsu_js).expect("Unable to write file");
}

fn build_with_args(args: Vec<String>, release: bool, run: bool) {
    // Lets run cargo build with the given args
    let mut command = std::process::Command::new("cargo");
    if run {
        command.arg("run");
    } else {
        command.arg("build");
    }
    if release {
        command.arg("--release");
    }
    command.args(args);
    let mut child = command.spawn().expect("failed to build");
    // lets wait for the build to complete
    child.wait().unwrap();
}

async fn start_webserver(path: &PathBuf) {
    // Let's start a webserver in the given directory
    let out_path = path.join("out");
    let directory = out_path.to_str().unwrap();
    // Add middleware to set Cache-Control header
    let add_no_cache_header = warp::any().map(|| {
        warp::reply::with_header(
            warp::reply(),
            "Cache-Control",
            "no-store, no-cache, must-revalidate, proxy-revalidate",
        )
    });

    let route = warp::get()
        .and(warp::fs::dir(str_static(directory.to_string())))
        .and(add_no_cache_header)
        .map(|reply, _| reply);

    done("Build done! Project live at http://localhost:5959");
    warp::serve(route).run(([127, 0, 0, 1], 5959)).await;
}

fn wasm_bindgen(path: &PathBuf, bin_name: &str) {
    let directory = Path::new(&path);
    let bin_path = directory.join(bin_name);
    let out_path = directory.join("out");
    let mut command = std::process::Command::new("wasm-bindgen");
    command.arg(format!("{}.wasm", bin_path.to_str().unwrap()));
    command.args(vec![
        "--target",
        "web",
        "--out-dir",
        out_path.to_str().unwrap(),
    ]);
    let mut child = command.spawn().expect("failed to build");
    // lets wait for the build to complete
    child.wait().unwrap();
}

fn str_static(s: String) -> &'static str {
    s.leak()
}

#[macro_export]
macro_rules! info {
    ($($t:tt)*) => ($crate::info(&format_args!($($t)*).to_string()))
}

pub fn info(msg: &str) {
    println!("{} {}", "[INFO]".bright_blue(), msg);
}

pub fn done(msg: &str) {
    println!("{} {}", "[DONE]".bright_green(), msg);
}
