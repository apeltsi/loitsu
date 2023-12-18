#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use loitsu::{load_scene_in_edit_mode, log};
use loitsu::ecs::ECS;
use loitsu::scene_management::Entity;
use loitsu::editor::{EventHandler, Event, ClientEvent};
use wasm_bindgen::prelude::*;
use loitsu::asset_management::get_file::get_file;
use wasm_bindgen_futures::spawn_local;
use loitsu::scripting::ScriptingSource;
use std::sync::{Arc, Mutex};
use crate::hierarchy::generate_hierarchy;

mod hierarchy;

static mut EVENT_HANDLER: Option<Arc<Mutex<EventHandler<loitsu::scripting::rune_runtime::RuneInstance>>>> = None;

fn main() {
    // When the server receives a request for LOITSU_MAIN_SCENE
    // it will automatically serve the correct scene from the asset folder

    spawn_local(async {
        let scene = get_file("LOITSU_MAIN_SCENE".to_string()).await;
        let scripts = get_file("LOITSU_ALL_SCRIPTS".to_string()).await;
        let scripts = scripts.unwrap();
        let scripts = String::from_utf8(scripts).unwrap();
        let scripts = serde_json::from_str::<Vec<String>>(&scripts).unwrap();
        let scene = scene.unwrap();
        // lets parse the Vec<u8> into a string
        let scene = String::from_utf8(scene).unwrap();
        let scene = loitsu::scene_management::Scene::from_json("Main Scene".to_string(), scene);
        let mut event_handler = EventHandler::new();
        event_handler.register_event_handler(Box::new(main_event_handler));
        let event_handler = Arc::new(Mutex::new(event_handler));
        unsafe { EVENT_HANDLER = Some(event_handler.clone()); }
        let scripts = scripts.iter().map(|script| {
            ScriptingSource {
                name: "Unknown Source".to_string(), 
                source: script.clone()
            }
        }).collect::<Vec<ScriptingSource>>();
        load_scene_in_edit_mode(event_handler, scene, scripts);
    });
}

fn main_event_handler<T>(ecs: &ECS<T>, event: &Event) where T: loitsu::scripting::ScriptingInstance {
    match event {
        Event::SceneLoaded(scene) => {
            log!("Scene loaded: {}", scene.name);
            let hierarchy = generate_hierarchy(ecs);
            set_hierarchy(serde_json::to_string(&hierarchy).unwrap());
            set_scene_name(scene.name.clone());
        },
        Event::EntityUpdated(name) => {
            log!("Entity updated: {}", name);
        },
        Event::EntitySelected(entity) => {
            log!("Selected entity {}", entity.name);
            select_entity(serde_json::to_string(&entity).unwrap());
        }
    }
}

#[wasm_bindgen]
pub fn request_select_entity(id: String) {
    let event_handler = unsafe { EVENT_HANDLER.as_ref().unwrap().clone() };
    let mut event_handler = event_handler.lock().unwrap();
    event_handler.emit_client(ClientEvent::SelectEntity(id));
}

#[wasm_bindgen]
extern "C" {
    fn set_scene_name(name: String);
    fn set_hierarchy(hierarchy: String);
    fn select_entity(entity: String);
}
