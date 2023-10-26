pub mod scripting;
pub mod rendering;
pub mod logging;
pub mod scene_management;
pub mod ecs;

use scripting::{ScriptingInstance, ScriptingSource};

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(feature = "scene_generation")]
pub fn build_scenes(scenes: Vec<(String, String)>, scripts: Vec<ScriptingSource>) -> Vec<scene_management::Scene> {
    let mut rune = scripting::rune_runtime::
        RuneInstance::new_with_sources(scripts).unwrap();
    let mut e = ecs::ECS::new();
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

/// Initializes the core systems of loitsu.
/// This function should be called before any other loitsu functions.
pub fn init_engine() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
    }
    log!("Loitsu core starting up...");
    let _rune = scripting::rune_runtime::
        RuneInstance::new_with_sources(
            vec![ScriptingSource{
                name: "main".to_string(),
                source: r#"
                    fn main() {
                        print("Hello, world!");
                    }
                "#.to_string()
            }
        
            ]).unwrap();
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
