use crate::{
    ecs,
    scene_management::{Entity, Property, Scene},
    scripting,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::{spawn_local, JsFuture};
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestInit, RequestMode};
pub struct EventHandler<T>
where
    T: scripting::ScriptingInstance,
{
    pub event_handlers: Vec<Box<dyn Fn(&ecs::ECS<T>, &Event)>>,
    client_events: Vec<ClientEvent>,
}

impl<T: scripting::ScriptingInstance> EventHandler<T> {
    pub fn new() -> EventHandler<T> {
        EventHandler {
            event_handlers: Vec::new(),
            client_events: Vec::new(),
        }
    }

    pub fn register_event_handler(&mut self, event_handler: Box<dyn Fn(&ecs::ECS<T>, &Event)>) {
        self.event_handlers.push(event_handler);
    }

    pub fn emit_client(&mut self, event: ClientEvent) {
        self.client_events.push(event);
    }

    pub fn poll_client_events(&mut self) -> Vec<ClientEvent> {
        let mut client_events = Vec::new();
        std::mem::swap(&mut self.client_events, &mut client_events);
        client_events
    }
}

/// This is a list of events that the editor backend can send to the client
pub enum Event {
    /// A scene has been loaded
    SceneLoaded(Scene),
    /// An entity has been updated
    EntityUpdated(String),
    /// An entity has been selected
    EntitySelected(Entity),
    /// The camera has changed (x, y, zoom)
    CameraChanged(f32, f32, f32),
    /// The selected entitys position on screen has changed. (x, y, width, height) where 1 is the width or height of the screen
    SelectedEntityPosition(f32, f32, f32, f32),
}

/// This is a list of events that the client can send to the editor backend
pub enum ClientEvent {
    /// A request to select an entity with the given uuid
    SelectEntity(String),
    /// A request to set the given property on the given entity
    SetComponentProperty {
        entity: String,
        component: String,
        field: String,
        property: Property,
    },
    /// A request to move the selected entity by the given amount
    MoveSelected(f32, f32),
    /// A request to save the current scene
    SaveScene,
}

#[cfg(target_arch = "wasm32")]
pub fn save_scene(scene: String) {
    #[cfg(feature = "editor")]
    crate::web::add_editor_loading_task("Saving Scene");
    // we need to send a post request to the server at /save_scene
    // with the scene data as the body
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&scene.into()));
    let request = Request::new_with_str_and_init("/save_scene", &opts).unwrap();
    request
        .headers()
        .set("Content-Type", "application/json")
        .unwrap();
    // now lets send the request
    let window = web_sys::window().unwrap();
    spawn_local(async move {
        let _ = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap();
        #[cfg(feature = "editor")]
        crate::web::remove_editor_loading_task("Saving Scene");
    });
}
