// on desktop platforms our job is to simply load the file from the filesystem
// on web platforms our job is to load the file from the server
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::asset_management::AssetError;

#[cfg(target_arch = "wasm32")]
static mut DIRECT_ASSET_PATH: &str = "./";

#[cfg(target_arch = "wasm32")]
pub fn set_direct_asset_path(path: String) {
    unsafe {
        DIRECT_ASSET_PATH = path.leak();
    }
}

#[cfg(target_arch = "wasm32")]
impl From<JsValue> for AssetError {
    fn from(value: JsValue) -> Self {
        AssetError::new(&format!("{:?}", value))
    }
}
#[cfg(target_arch = "wasm32")]
pub async fn get_file(path: String) -> Result<Vec<u8>, AssetError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = format!("{}{}", unsafe { DIRECT_ASSET_PATH }, path);
    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let response: Response = resp_value.dyn_into()?;

    // now lets turn the response into a byte vector
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    let buffer = js_sys::Uint8Array::new(&buffer);
    let mut file_contents = Vec::with_capacity(buffer.length() as usize);

    for i in 0..buffer.length() {
        file_contents.push(buffer.get_index(i));
    }
    Ok(file_contents)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_file(path: String) -> Result<Vec<u8>, AssetError> {
    let mut path_buf = std::env::current_exe().unwrap();
    path_buf.pop();
    path_buf.push(path);
    let data = std::fs::read(path_buf)?;
    Ok(data)
}
