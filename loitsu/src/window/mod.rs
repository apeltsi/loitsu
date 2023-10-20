// WASM
#[cfg(target_arch = "wasm32")]
pub mod web;

// Desktop
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
