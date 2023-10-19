use loitsu::init_engine;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    init_engine();
}
