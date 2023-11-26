// WASM
#[cfg(target_arch = "wasm32")]
pub mod web;

// Desktop
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;

pub mod core;

pub mod drawable;
pub mod shader;
pub mod vertex;
