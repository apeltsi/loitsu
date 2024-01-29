use std::env;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../web/dist/*");
    // lets copy the web directory into the target directory
    let path_buf = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut path_buf = PathBuf::from(path_buf);
    path_buf.pop();
    path_buf.push("web");
    path_buf.push("dist");
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut target_path_buf = PathBuf::from(out_dir);
    target_path_buf.pop();
    target_path_buf.pop();
    target_path_buf.pop();
    target_path_buf.push("editor_assets");
    copy_dir(&path_buf, &target_path_buf).unwrap();
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    println!("copying {:?} to {:?}", src, dst);
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
