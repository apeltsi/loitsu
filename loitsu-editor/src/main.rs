#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use loitsu::{load_scene_in_edit_mode, log};
use loitsu::ecs::ECS;
use loitsu::editor::{EventHandler, Event};
use wasm_bindgen::prelude::*;
fn main() {
    let scene = loitsu::scene_management::Scene::new("New Scene".to_string());
    let scripts = Vec::new();
    let mut event_handler = EventHandler::new();
    event_handler.register_event_handler(Box::new(main_event_handler));
    load_scene_in_edit_mode(event_handler, scene, scripts);
}

fn main_event_handler<T>(ecs: &ECS<T>, event: &Event) where T: loitsu::scripting::ScriptingInstance {
    match event {
        Event::SceneLoaded(scene) => {
            log!("Scene loaded: {}", scene.name);
            set_scene_name(scene.name.clone());
        },
        Event::EntityUpdated(name) => {
            log!("Entity updated: {}", name);
        }
    }
}
#[wasm_bindgen]
extern "C" {
    fn set_scene_name(name: String);
}
