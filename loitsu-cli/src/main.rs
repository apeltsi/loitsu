use clap::{Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use warp::Filter;
use cargo_toml::{Manifest, Dependency};

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
        release: bool
    },
    Run {
        #[arg(short, long, default_value = "")]
        target: String,
        #[arg(short, long, default_value = "false")]
        release: bool
    }
}
#[tokio::main]
async fn main() {
    println!("Loitsu Dev Tools");

    let args = Cli::parse();

    match args.command {
        Commands::Build { target, release } => build(&target, release, false).await,
        Commands::Run { target, release } => build(&target, release, true).await,
    }
}

async fn build(target: &str, release: bool, run: bool) {

    if target == "web" {
        println!("Building for web");
        // Now we can build the target
        build_with_args(vec!["--target=wasm32-unknown-unknown".to_string()], release, false);
        // Lets read the name of the app we just built, this can be determined from the Cargo.toml file
        let mut path = std::env::current_dir().unwrap();
        path.push("Cargo.toml");
        let manifest = Manifest::from_path(path).unwrap();
        let package_name = manifest.package.unwrap().name;

        let loitsu_version = match &manifest.dependencies["loitsu"] {
            Dependency::Simple(ver) => ver.to_owned(),
            Dependency::Inherited(_inherited) => "Unknown".to_string(),
            Dependency::Detailed(detail) => {
                let ver = &detail.version;
                match ver {
                    Some(version) => version.to_owned(),
                    None => "Unknown".to_string()
                }
            }
        };
        println!("Creating player...");
        // Lets copy the web player files
        generate_player_files(&package_name, &loitsu_version, release);
        println!("Build Done!");
        if run {
            let mut path = std::env::current_dir().unwrap();
            
            path.push("target");
            path.push("wasm32-unknown-unknown");
            if release {
                path.push("release");
            } else {
                path.push("debug");
            }


            start_webserver(path.to_str().unwrap()).await;
        }
    } else if target == "" {
        // Now we can build the target
        println!("Building for native");
        build_with_args(vec![], release, run);
    } else {
        panic!("Unsupported target: {}", target);
    }
}

fn generate_player_files(app_name: &str, loitsu_version: &str, release: bool) {
    // First lets load the player html file located in /player
    let raw_player_html = include_str!("../player/index.html");
    let player_html = raw_player_html.replace("{APP_NAME}", &app_name).replace("{LOITSU_VERSION}", &loitsu_version);

    // Now lets write the html file to the output directory
    
    let mut path = std::env::current_dir().unwrap();

    path.push("target");
    path.push("wasm32-unknown-unknown");
    if release {
        path.push("release");
    } else {
        path.push("debug");
    }
    let out_str = path.to_str().unwrap();
    let out_dir = Path::new(&out_str);
    let dest_path = &out_dir.join("index.html");
    let mut f = File::create(&dest_path).unwrap();
    println!("Writing player.html to {}", dest_path.to_str().unwrap());
    f.write_all(player_html.as_bytes()).unwrap();

    // Finally lets copy the loitsu logo to the output directory
    let logo_bytes = include_bytes!("../player/loitsu.png");
    let dest_path = Path::new(&out_dir).join("loitsu.png");
    fs::write(&dest_path, logo_bytes).expect("Unable to write file")
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
    command.env("RUSTFLAGS", "--cfg=web_sys_unstable_apis");
    let mut child = command.spawn().expect("failed to build");
    // lets wait for the build to complete
    child.wait().unwrap();
}

async fn start_webserver(directory: &str) {
    // Let's start a webserver in the given directory

    // Add middleware to set Cache-Control header
    let add_no_cache_header = warp::any().map(|| {
        warp::reply::with_header(warp::reply(), "Cache-Control", "no-store, no-cache, must-revalidate, proxy-revalidate")
    });

    let route = warp::get()
        .and(warp::fs::dir(str_static(directory.to_string())))
        .and(add_no_cache_header)
        .map(|reply, _| reply);

    println!("Project live at http://localhost:5959");
    warp::serve(route).run(([127, 0, 0, 1], 5959)).await;
}

fn str_static(s: String) -> &'static str {
    s.leak()
}
