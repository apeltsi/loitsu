use crate::{
    ecs,
    scene_management::{Entity, Property, Scene},
    scripting,
};
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
    SetComponentProperty {
        entity: String,
        component: String,
        field: String,
        property: Property,
    },
    MoveSelected(f32, f32),
}
