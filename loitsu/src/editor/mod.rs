use crate::{ecs, scripting, scene_management::Scene};
pub struct EventHandler<T> where T: scripting::ScriptingInstance { 
    pub event_handlers: Vec<Box<dyn Fn(&ecs::ECS<T>, &Event)>>,
}

impl<T: scripting::ScriptingInstance> EventHandler<T> {
    pub fn new() -> EventHandler<T> {
        EventHandler {
            event_handlers: Vec::new(),
        }
    }

    pub fn register_event_handler(&mut self, event_handler: Box<dyn Fn(&ecs::ECS<T>, &Event)>) {
        self.event_handlers.push(event_handler);
    }
}

pub enum Event {
    SceneLoaded(Scene),
    EntityUpdated(String),
}
