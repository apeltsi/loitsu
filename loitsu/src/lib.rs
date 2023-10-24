mod scripting;
mod rendering;
mod logging;

use scripting::ScriptingInstance;

#[cfg(target_arch = "wasm32")]
mod web;

/// Initializes the core systems of loitsu.
/// This function should be called before any other loitsu functions.
pub fn init_engine() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
    }
    log!("Loitsu core starting up...");
    let mut rune = scripting::rune::RuneInstance::new().unwrap();
    rune.add_script("test", "test", "fn main() { println(\"Hello, world!\"); }").unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rendering::desktop;
        desktop::init_window();
    }

    #[cfg(target_arch = "wasm32")]
    {
        use rendering::web;
        web::init_view();
    }
}

#[cfg(test)]
mod tests {

}
