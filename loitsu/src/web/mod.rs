use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn set_status(status: u32);
}

pub fn update_loading_status(status: u32) {
    set_status(status);
}
