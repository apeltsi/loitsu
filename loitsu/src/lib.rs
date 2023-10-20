mod scripting;
mod window;


use scripting::ScriptingInstance;


/// Initializes the core systems of loitsu.
/// This function should be called before any other loitsu functions.
pub fn init_engine() {
    println!("Loitsu core starting up...");
    let mut lua = scripting::lua::LuaInstance::new().unwrap();
    {
        lua.load_script("LOITSU_LUA", "print(\"Loitsu Lua Runtime is online\");").unwrap();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use window::desktop;
        desktop::init_window();
    }

    #[cfg(target_arch = "wasm32")]
    {
        use window::web;
        web::init_view();
    }
}

#[cfg(test)]
mod tests {

}
