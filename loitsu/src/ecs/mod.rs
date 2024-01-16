use std::cell::RefCell;
use std::rc::Rc;

#[cfg(not(feature = "scene_generation"))]
use crate::asset_management::ASSET_MANAGER;

use crate::scene_management::{Component, Entity, Property, Scene};
use crate::scripting::{EntityUpdate, ScriptingData, ScriptingInstance};
use bitflags::bitflags;
#[cfg(feature = "scene_generation")]
use serde_json::{Map, Number, Value};
#[cfg(feature = "editor")]
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct ECS<T>
where
    T: ScriptingInstance,
{
    pub active_scene: Scene,
    pub static_scene: Option<Scene>,
    runtime_entities: Vec<Rc<RefCell<RuntimeEntity<T>>>>,
    #[cfg(feature = "editor")]
    event_handler: Arc<Mutex<crate::editor::EventHandler<T>>>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ComponentFlags: u32 {
        const EMPTY =           0b00000000;
        const BUILD =           0b00000001;
        const FRAME =           0b00000010;
        const LATE_FRAME =      0b00000100;
        const TICK =            0b00001000;
        const START =           0b00010000;
        const DESTROY =         0b00100000;
        #[cfg(feature = "editor")]
        const EDITOR_START =    0b01000000;
        #[cfg(feature = "editor")]
        const EDITOR_DESTROY =  0b10000000;
        #[cfg(feature = "editor")]
        const EDITOR_UPDATE =    0b00000001_00000000;
    }
}

#[cfg_attr(
    feature = "scene_generation",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub enum Transform {
    Transform2D {
        position: (f32, f32),
        rotation: f32,
        scale: (f32, f32),
        r#static: bool,
        changed_frame: u64,
        has_changed: bool,
    },
    RectTransform {
        // TODO: Implement this :D
        position: (f32, f32),
        changed_frame: u64,
        has_changed: bool,
    },
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Transform::Transform2D {
                    position: (x1, y1),
                    rotation: r1,
                    scale: (sx1, sy1),
                    ..
                },
                Transform::Transform2D {
                    position: (x2, y2),
                    rotation: r2,
                    scale: (sx2, sy2),
                    ..
                },
            ) => x1 == x2 && y1 == y2 && r1 == r2 && sx1 == sx2 && sy1 == sy2,
            (
                Transform::RectTransform {
                    position: (x1, y1), ..
                },
                Transform::RectTransform {
                    position: (x2, y2), ..
                },
            ) => x1 == x2 && y1 == y2,
            _ => false,
        }
    }
}

impl Transform {
    #[cfg(feature = "scene_generation")]
    pub fn to_json(self) -> Value {
        match self {
            Transform::Transform2D {
                position,
                rotation,
                scale,
                r#static,
                ..
            } => {
                let mut map = Map::new();
                map.insert(
                    "position".to_string(),
                    Value::Array(vec![
                        Value::Number(Number::from_f64(position.0 as f64).unwrap()),
                        Value::Number(Number::from_f64(position.1 as f64).unwrap()),
                    ]),
                );
                map.insert(
                    "rotation".to_string(),
                    Value::Number(Number::from_f64(rotation as f64).unwrap()),
                );
                map.insert(
                    "scale".to_string(),
                    Value::Array(vec![
                        Value::Number(Number::from_f64(scale.0 as f64).unwrap()),
                        Value::Number(Number::from_f64(scale.1 as f64).unwrap()),
                    ]),
                );
                map.insert("static".to_string(), Value::Bool(r#static));
                Value::Object(map)
            }
            Transform::RectTransform { position, .. } => {
                let mut map = Map::new();
                map.insert(
                    "position".to_string(),
                    Value::Array(vec![
                        Value::Number(Number::from_f64(position.0 as f64).unwrap()),
                        Value::Number(Number::from_f64(position.1 as f64).unwrap()),
                    ]),
                );
                Value::Object(map)
            }
        }
    }
    #[cfg(feature = "scene_generation")]
    pub fn from_json(json: &Map<String, Value>) -> Transform {
        let position = json["position"].as_array().unwrap();
        let position = (
            position[0].as_f64().unwrap() as f32,
            position[1].as_f64().unwrap() as f32,
        );
        let rotation = json["rotation"].as_f64().unwrap() as f32;
        let scale = json["scale"].as_array().unwrap();
        let scale = (
            scale[0].as_f64().unwrap() as f32,
            scale[1].as_f64().unwrap() as f32,
        );
        let r#static = json["static"].as_bool().unwrap();
        Transform::Transform2D {
            position,
            rotation,
            scale,
            r#static,
            changed_frame: 0,
            has_changed: true,
        }
    }

    pub fn check_changed(&mut self, frame_num: u64) -> bool {
        match self {
            Transform::Transform2D {
                changed_frame,
                has_changed,
                ..
            } => {
                if *changed_frame == frame_num {
                    return true;
                } else if *has_changed {
                    *changed_frame = frame_num;
                    *has_changed = false;
                    return true;
                } else {
                    return false;
                }
            }
            Transform::RectTransform {
                changed_frame,
                has_changed,
                ..
            } => {
                if *changed_frame == frame_num {
                    return true;
                } else if *has_changed {
                    *changed_frame = frame_num;
                    *has_changed = false;
                    return true;
                } else {
                    return false;
                }
            }
        }
    }
}

#[allow(dead_code)]
pub struct RuntimeEntity<T>
where
    T: ScriptingInstance,
{
    name: String,
    id: Uuid,
    pub components: Vec<RuntimeComponent<T>>,
    entity_proto: Entity,
    pub children: Vec<Rc<RefCell<RuntimeEntity<T>>>>,
    pub component_flags: ComponentFlags, // this is the union of all the component flags, so we can quickly check if we need to run a method
    pub transform: Rc<RefCell<Transform>>,
    pub is_new: bool,
}

impl<T> RuntimeEntity<T>
where
    T: ScriptingInstance,
{
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_id(&self) -> String {
        self.id.to_string()
    }

    pub fn get_uuid(&self) -> Uuid {
        self.id
    }

    pub fn get_component_mut(&mut self, id: &str) -> Option<&mut RuntimeComponent<T>> {
        for component in self.components.iter_mut() {
            if component.component_proto.id == id {
                return Some(component);
            }
        }
        None
    }
}

#[allow(dead_code)]
pub struct RuntimeComponent<T>
where
    T: ScriptingInstance,
{
    pub data: T::Data,
    pub component_proto: Component,
    pub flags: ComponentFlags,
}

impl<T: ScriptingInstance> ECS<T> {
    #[cfg(not(feature = "editor"))]
    pub fn new() -> ECS<T> {
        ECS {
            active_scene: Scene::new("INITIAL_SCENE".to_string()),
            static_scene: None,
            runtime_entities: Vec::new(),
        }
    }

    #[cfg(feature = "editor")]
    pub fn new(event_handler: Arc<Mutex<crate::editor::EventHandler<T>>>) -> ECS<T> {
        ECS {
            active_scene: Scene::new("INITIAL_SCENE".to_string()),
            static_scene: None,
            runtime_entities: Vec::new(),
            event_handler,
        }
    }

    pub fn load_scene(&mut self, scene: Scene, scripting: &mut T) {
        self.active_scene = scene.clone();

        self.runtime_entities = init_entities(scene.clone().entities, scripting);
        #[cfg(feature = "editor")]
        self.emit(crate::editor::Event::SceneLoaded(scene));

        // next up we'll have to figure out how to load our assets
        // lets start by requesting the appropriate shards
        #[cfg(not(feature = "scene_generation"))]
        ASSET_MANAGER
            .lock()
            .unwrap()
            .request_shards(scene.shards.clone());
    }

    #[cfg(feature = "editor")]
    pub fn emit(&mut self, event: crate::editor::Event) {
        for event_handler in &self.event_handler.lock().unwrap().event_handlers {
            event_handler(&self, &event);
        }
    }

    #[cfg(feature = "editor")]
    pub fn poll_client_events(&mut self) -> Vec<crate::editor::ClientEvent> {
        self.event_handler.lock().unwrap().poll_client_events()
    }

    pub fn get_entity(&self, id: &str) -> Option<Rc<RefCell<RuntimeEntity<T>>>> {
        for runtime_entity in self.runtime_entities.iter() {
            if runtime_entity.borrow().get_id() == id {
                return Some(runtime_entity.clone());
            }
        }
        None
    }

    pub fn run_build_step(&mut self, scripting: &mut T) {
        self.run_component_methods(scripting, ComponentFlags::BUILD);
    }

    pub fn run_frame(
        &mut self,
        scripting: &mut T,
    ) -> Vec<(Rc<RefCell<Transform>>, Vec<EntityUpdate>)> {
        self.run_component_methods(scripting, ComponentFlags::FRAME)
    }

    pub fn run_component_methods(
        &mut self,
        scripting: &mut T,
        method: ComponentFlags,
    ) -> Vec<(Rc<RefCell<Transform>>, Vec<EntityUpdate>)> {
        // Lets iterate over the entities and run the build step on each component
        scripting.run_component_methods::<T>(self.runtime_entities.as_mut_slice(), method)
    }

    pub fn clear(&mut self) {
        self.active_scene = Scene::new("INITIAL_SCENE".to_string());
        self.static_scene = None;
        self.runtime_entities = Vec::new();
    }

    pub fn as_scene(&self) -> Scene {
        Scene {
            name: self.active_scene.name.clone(),
            entities: self
                .runtime_entities
                .iter()
                .map(|runtime_entity| runtime_entity.borrow().as_entity())
                .collect(),
            required_assets: Vec::new(),
            shards: Vec::new(),
        }
    }

    pub fn get_runtime_entities(&self) -> &Vec<Rc<RefCell<RuntimeEntity<T>>>> {
        &self.runtime_entities
    }
}

impl<T: ScriptingInstance> RuntimeEntity<T> {
    pub fn as_entity(&self) -> Entity {
        Entity {
            name: self.name.clone(),
            id: self.id.to_string(),
            components: self
                .components
                .iter()
                .map(|runtime_component| runtime_component.as_component())
                .collect(),
            children: self
                .children
                .iter()
                .map(|runtime_entity| runtime_entity.borrow().as_entity())
                .collect(),
            transform: self.transform.borrow().clone(),
        }
    }
}

impl<T: ScriptingInstance> RuntimeComponent<T> {
    pub fn as_component(&self) -> Component {
        self.data.to_component_proto(&self.component_proto).unwrap()
    }

    pub fn set_property(&mut self, field: &str, property: Property) {
        let _ = self.data.set_property(field, property);
    }
}

fn init_entities<T>(
    proto_entities: Vec<Entity>,
    scripting: &mut T,
) -> Vec<Rc<RefCell<RuntimeEntity<T>>>>
where
    T: ScriptingInstance,
{
    // Lets recursively iterate over the entities and create a runtime entity for each one
    let mut runtime_entities = Vec::new();
    for proto_entity in proto_entities {
        let mut runtime_entity = RuntimeEntity {
            name: proto_entity.name.clone(),
            id: Uuid::parse_str(proto_entity.id.as_str()).unwrap(),
            components: Vec::new(),
            entity_proto: proto_entity.clone(),
            children: init_entities(proto_entity.children.clone(), scripting),
            component_flags: ComponentFlags::EMPTY,
            transform: Rc::new(RefCell::new(proto_entity.transform.clone())),
            is_new: true,
        };

        for proto_component in runtime_entity.entity_proto.components.clone() {
            let flags = scripting.get_component_flags(proto_component.name.as_str());
            runtime_entity.component_flags |= flags;
            let runtime_component = RuntimeComponent {
                data: ScriptingData::from_component_proto(proto_component.clone(), scripting)
                    .unwrap(),
                component_proto: proto_component,
                flags,
            };
            runtime_entity.components.push(runtime_component);
        }
        runtime_entities.push(Rc::new(RefCell::new(runtime_entity)));
    }
    runtime_entities
}
