pub mod scripting;
pub mod rendering;
pub mod logging;
pub mod scene_management;
pub mod ecs;
pub mod asset_management;
pub mod util;
pub mod input;
#[cfg(feature = "editor")]
pub mod editor;
use scripting::ScriptingInstance;

#[cfg_attr(feature = "json_preference_parse", derive(serde::Deserialize))]
#[derive(Clone, bitcode::Decode, bitcode::Encode)]
pub struct Preferences {
    pub default_scene: String
}

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(feature = "scene_generation")]
pub fn build_scenes(scenes: Vec<(String, String)>, scripts: Vec<scripting::ScriptingSource>) -> Vec<scene_management::Scene> {
    let mut rune = scripting::rune_runtime::
        RuneInstance::new_with_sources(scripts).unwrap();
    #[cfg(not(feature = "editor"))]
    let mut e = ecs::ECS::new();
    #[cfg(feature = "editor")]
    let mut e = ecs::ECS::new(std::sync::Arc::new(std::sync::Mutex::new(editor::EventHandler::new())));
    let mut generated_scenes = Vec::new();
    for scene in scenes {
        let scene = scene_management::Scene::from_json(scene.0, scene.1);
        e.load_scene(scene, &mut rune);
        e.run_build_step(&mut rune);
        let mut scene = e.as_scene();
        e.clear();
        scene.required_assets = unsafe { scripting::rune_runtime::get_required_assets() };
        unsafe { scripting::rune_runtime::clear_required_assets() };
        generated_scenes.push(scene);
    }
    generated_scenes
}

#[cfg(target_arch = "wasm32")]
#[cfg(feature = "editor")]
pub fn load_scene_in_edit_mode(event_handler: std::sync::Arc<std::sync::Mutex<editor::EventHandler<scripting::rune_runtime::RuneInstance>>>, scene: scene_management::Scene, scripts: Vec<scripting::ScriptingSource>) {
    log!("Loading scene in edit mode...");
    web::add_editor_loading_task("Starting ECS...");
    let mut rune = scripting::rune_runtime::RuneInstance::new_with_sources(scripts).unwrap();
    let mut e = ecs::ECS::new(event_handler);
    e.load_scene(scene, &mut rune);
    web::remove_editor_loading_task("Starting ECS...");
    web::add_editor_loading_task("Starting render pipeline...");
    log!("ECS initialized, starting render loop...");
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rendering::desktop;
        desktop::init_window(rune, e);
    }

    #[cfg(target_arch = "wasm32")]
    {
        use rendering::web;
        web::init_view(rune, e);
    }
}

/// Initializes the core systems of loitsu.
/// This function should be called before any other loitsu functions.
#[cfg(not(feature = "editor"))]
pub fn init_engine() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
    }

    log!("Loitsu core starting up...");
    let rune = scripting::rune_runtime::RuneInstance::new_uninitialized().unwrap();
    let ecs = ecs::ECS::<scripting::rune_runtime::RuneInstance>::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        use rendering::desktop;
        desktop::init_window(rune, ecs);
    }

    #[cfg(target_arch = "wasm32")]
    {
        use rendering::web;
        web::init_view(rune, ecs);
    }
}

#[cfg(test)]
mod tests {

}
