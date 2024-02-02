use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(not(feature = "scene_generation"))]
use crate::asset_management::ASSET_MANAGER;

use crate::scene_management::{Component, Entity, Property, Scene};
use crate::scripting::{EntityUpdate, ScriptingData, ScriptingInstance};
use bitflags::bitflags;
#[cfg(feature = "scene_generation")]
use serde_json::{Map, Number, Value};
use std::sync::{Arc, Mutex};

pub struct ECS<T>
where
    T: ScriptingInstance,
{
    pub active_scene: Scene,
    pub static_scene: Option<Scene>,
    runtime_entities: Vec<Rc<RefCell<RuntimeEntity<T>>>>,
    entity_lookup: HashMap<u32, Rc<RefCell<RuntimeEntity<T>>>>,
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
    },
    RectTransform {
        // TODO: Implement this :D
        position: (f32, f32),
    },
}

pub struct RuntimeTransform {
    pub transform: Transform,
    parent: Option<Arc<Mutex<RuntimeTransform>>>,
    changed_frame: u64,
    evaluated_position: (f32, f32),
    evaluated_rotation: f32,
    evaluated_scale: (f32, f32),
    evaluated_frame: u64,
    pub has_changed: bool,
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
        }
    }
}

impl RuntimeTransform {
    pub fn new(transform: Transform) -> RuntimeTransform {
        RuntimeTransform {
            transform,
            parent: None,
            changed_frame: u64::MAX,
            has_changed: true,
            evaluated_position: (0.0, 0.0),
            evaluated_rotation: 0.0,
            evaluated_scale: (1.0, 1.0),
            evaluated_frame: u64::MAX,
        }
    }

    pub fn check_changed(&mut self, frame_num: u64) -> bool {
        if self.parent.is_some() {
            let mut parent = self.parent.as_ref().unwrap().lock().unwrap();
            if parent.check_changed(frame_num) {
                self.has_changed = true;
            }
        }
        if self.changed_frame == frame_num {
            return true;
        } else if self.has_changed {
            self.changed_frame = frame_num;
            self.has_changed = false;
            return true;
        } else {
            return false;
        }
    }

    pub fn get_parent(&self) -> Option<Arc<Mutex<RuntimeTransform>>> {
        self.parent.clone()
    }

    fn set_parent(&mut self, parent: Option<Arc<Mutex<RuntimeTransform>>>) {
        self.parent = parent;
    }

    pub fn eval_transform_mat(&mut self, frame_num: u64) -> [[f32; 4]; 4] {
        let (position, rotation, scale) = self.eval_transform(frame_num);
        let sin = rotation.sin();
        let cos = rotation.cos();
        [
            [scale.0 * cos, scale.1 * -sin, 0.0, position.0],
            [scale.0 * sin, scale.1 * cos, 0.0, position.1],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    pub fn eval_transform(&mut self, frame_num: u64) -> ((f32, f32), f32, (f32, f32)) {
        if self.evaluated_frame == frame_num {
            return (
                self.evaluated_position,
                self.evaluated_rotation,
                self.evaluated_scale,
            );
        }
        let (position, rotation, scale) = match &self.transform {
            Transform::Transform2D {
                position,
                rotation,
                scale,
                ..
            } => (*position, *rotation, *scale),
            Transform::RectTransform { position, .. } => (*position, 0.0, (1.0, 1.0)),
        };
        let mut position = (position.0, position.1);
        let mut rotation = rotation;
        let mut scale = (scale.0, scale.1);
        if let Some(parent) = &self.parent {
            let mut parent = parent.lock().unwrap();
            let (parent_position, parent_rotation, parent_scale) = parent.eval_transform(frame_num);
            position.0 += parent_position.0;
            position.1 += parent_position.1;
            rotation += parent_rotation;
            scale.0 *= parent_scale.0;
            scale.1 *= parent_scale.1;
        }
        self.evaluated_position = position;
        self.evaluated_rotation = rotation;
        self.evaluated_scale = scale;
        self.evaluated_frame = frame_num;
        (position, rotation, scale)
    }
}

#[allow(dead_code)]
pub struct RuntimeEntity<T>
where
    T: ScriptingInstance,
{
    name: String,
    id: u32,
    pub components: Vec<RuntimeComponent<T>>,
    entity_proto: Entity,
    pub children: Vec<Rc<RefCell<RuntimeEntity<T>>>>,
    pub component_flags: ComponentFlags, // this is the union of all the component flags, so we can quickly check if we need to run a method
    pub transform: Arc<Mutex<RuntimeTransform>>,
    pub is_new: bool,
}

impl<T> RuntimeEntity<T>
where
    T: ScriptingInstance,
{
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_component_mut(&mut self, id: u32) -> Option<&mut RuntimeComponent<T>> {
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
            entity_lookup: HashMap::new(),
        }
    }

    #[cfg(feature = "editor")]
    pub fn new(event_handler: Arc<Mutex<crate::editor::EventHandler<T>>>) -> ECS<T> {
        ECS {
            active_scene: Scene::new("INITIAL_SCENE".to_string()),
            static_scene: None,
            runtime_entities: Vec::new(),
            entity_lookup: HashMap::new(),
            event_handler,
        }
    }

    pub fn load_scene(&mut self, scene: Scene, scripting: &mut T) {
        self.active_scene = scene.clone();

        (self.runtime_entities, self.entity_lookup) =
            init_entities(scene.clone().entities, scripting, None);
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

    pub fn get_entity(&self, id: u32) -> Option<Rc<RefCell<RuntimeEntity<T>>>> {
        self.entity_lookup.get(&id).map(|entity| entity.clone())
    }

    pub fn run_build_step(&mut self, scripting: &mut T) {
        self.run_component_methods(scripting, ComponentFlags::BUILD);
    }

    pub fn run_frame(
        &mut self,
        scripting: &mut T,
    ) -> Vec<(Arc<Mutex<RuntimeTransform>>, Vec<EntityUpdate>)> {
        self.run_component_methods(scripting, ComponentFlags::FRAME)
    }

    pub fn run_component_methods(
        &mut self,
        scripting: &mut T,
        method: ComponentFlags,
    ) -> Vec<(Arc<Mutex<RuntimeTransform>>, Vec<EntityUpdate>)> {
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
    /// Returns a flat list of all entities in the scene
    pub fn get_all_runtime_entities_flat(&self) -> Vec<Rc<RefCell<RuntimeEntity<T>>>> {
        let mut entities = Vec::new();
        for runtime_entity in &self.runtime_entities {
            entities.push(runtime_entity.clone());
            entities.append(&mut runtime_entity.borrow().get_all_runtime_entities());
        }
        entities
    }
}

impl<T: ScriptingInstance> RuntimeEntity<T> {
    pub fn as_entity(&self) -> Entity {
        Entity {
            name: self.name.clone(),
            id: self.id,
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
            transform: self.transform.lock().unwrap().transform.clone(),
        }
    }

    pub fn get_all_runtime_entities(&self) -> Vec<Rc<RefCell<RuntimeEntity<T>>>> {
        let mut entities = Vec::new();
        for runtime_entity in &self.children {
            entities.push(runtime_entity.clone());
            entities.append(&mut runtime_entity.borrow().get_all_runtime_entities());
        }
        entities
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
    parent_transform: Option<Arc<Mutex<RuntimeTransform>>>,
) -> (
    Vec<Rc<RefCell<RuntimeEntity<T>>>>,
    HashMap<u32, Rc<RefCell<RuntimeEntity<T>>>>,
)
where
    T: ScriptingInstance,
{
    // Lets recursively iterate over the entities and create a runtime entity for each one
    let mut runtime_entities = Vec::new();
    let mut entity_lookup = HashMap::new();
    for proto_entity in proto_entities {
        let mut transform = RuntimeTransform::new(proto_entity.transform.clone());
        transform.set_parent(parent_transform.clone());
        let transform = Arc::new(Mutex::new(transform));
        let children = init_entities(
            proto_entity.children.clone(),
            scripting,
            Some(transform.clone()),
        );
        let mut runtime_entity = RuntimeEntity {
            name: proto_entity.name.clone(),
            id: proto_entity.id,
            components: Vec::new(),
            entity_proto: proto_entity.clone(),
            children: children.0,
            component_flags: ComponentFlags::EMPTY,
            transform,
            is_new: true,
        };
        entity_lookup.extend(children.1);

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
        let runtime_entity = Rc::new(RefCell::new(runtime_entity));
        runtime_entities.push(runtime_entity.clone());
        let id = runtime_entity.borrow().id;
        entity_lookup.insert(id, runtime_entity);
    }
    (runtime_entities, entity_lookup)
}
