use std::cell::RefCell;
use std::rc::Rc;

#[cfg(not(feature = "scene_generation"))]
use crate::asset_management::ASSET_MANAGER;
use crate::scene_management::{Scene, Entity, Component};
use crate::scripting::{ScriptingData, ScriptingInstance};
use bitflags::bitflags;

pub struct ECS<T> where T: ScriptingInstance {
    pub active_scene: Scene,
    pub static_scene: Option<Scene>,
    runtime_entities: Vec<RuntimeEntity<T>>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ComponentFlags: u32 {
        const EMPTY =       0b00000000;
        const BUILD =       0b00000001;
        const FRAME =       0b00000010;
        const LATE_FRAME =  0b00000100;
        const TICK =        0b00001000;
        const START =       0b00010000;
        const DESTROY =     0b00100000;
    }
}
#[derive(Debug, Clone)]
pub enum Transform {
    Transform2D {
        position: (f32, f32),
        rotation: f32,
        scale: (f32, f32),
        r#static: bool
    },
    RectTransform {
        // TODO: Implement this :D
        position: (f32, f32)
    }
}

#[allow(dead_code)]
pub struct RuntimeEntity<T> where T: ScriptingInstance {
    name: String,
    id: String,
    pub components: Vec<RuntimeComponent<T>>,
    entity_proto: Entity,
    pub children: Vec<RuntimeEntity<T>>,
    pub component_flags: ComponentFlags, // this is the union of all the component flags, so we can quickly check if we need to run a method
    pub transform: Rc<RefCell<Transform>>,
    pub is_new: bool
}

impl<T> RuntimeEntity<T> where T: ScriptingInstance {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
}

#[allow(dead_code)]
pub struct RuntimeComponent<T> where T: ScriptingInstance {
    pub data: T::Data,
    pub component_proto: Component,
    pub flags: ComponentFlags,
}

impl<T: ScriptingInstance> ECS<T> {
    pub fn new() -> ECS<T> {
        ECS {
            active_scene: Scene::new("INITIAL_SCENE".to_string()),
            static_scene: None,
            runtime_entities: Vec::new(),
        }
    }

    pub fn load_scene(&mut self, scene: Scene, scripting: &mut T) {
        self.active_scene = scene.clone();
        self.runtime_entities = init_entities(scene.entities, scripting);

        // next up we'll have to figure out how to load our assets
        // lets start by requesting the appropriate shards
        
        #[cfg(not(feature = "scene_generation"))]
        ASSET_MANAGER.lock().unwrap().request_shards(scene.shards.clone());
    }

    pub fn run_build_step(&mut self, scripting: &mut T) {
        self.run_component_methods(scripting, ComponentFlags::BUILD);
    }

    pub fn run_frame(&mut self, scripting: &mut T) {
        self.run_component_methods(scripting, ComponentFlags::FRAME);
    }

    fn run_component_methods(&mut self, scripting: &mut T, method: ComponentFlags) {
        // Lets iterate over the entities and run the build step on each component
        scripting.run_component_methods::<T>(self.runtime_entities.as_mut_slice(), method);
    }

    pub fn clear(&mut self) {
        self.active_scene = Scene::new("INITIAL_SCENE".to_string());
        self.static_scene = None;
        self.runtime_entities = Vec::new();
    }

    #[cfg(feature = "scene_generation")]
    pub fn as_scene(&self) -> Scene {
        Scene {
            name: self.active_scene.name.clone(),
            entities: self.runtime_entities.iter().map(|runtime_entity| runtime_entity.as_entity()).collect(),
            required_assets: Vec::new(),
            shards: Vec::new(),
        }
    }
}

impl<T: ScriptingInstance> RuntimeEntity<T> {
    #[cfg(feature = "scene_generation")]
    pub fn as_entity(&self) -> Entity {
        Entity {
            name: self.name.clone(),
            id: self.id.clone(),
            components: self.components.iter().map(|runtime_component| runtime_component.as_component()).collect(),
            children: self.children.iter().map(|runtime_entity| runtime_entity.as_entity()).collect(),
        }
    }
}

impl<T: ScriptingInstance> RuntimeComponent<T> {
    #[cfg(feature = "scene_generation")]
    pub fn as_component(&self) -> Component {
        self.data.to_component_proto(&self.component_proto).unwrap()
    }
}

fn init_entities<T>(proto_entities: Vec<Entity>, scripting: &mut T) -> Vec<RuntimeEntity<T>> where T: ScriptingInstance {
    // Lets recursively iterate over the entities and create a runtime entity for each one
    let mut runtime_entities = Vec::new();
    for proto_entity in proto_entities {
        let mut runtime_entity = RuntimeEntity {
            name: proto_entity.name.clone(),
            id: proto_entity.id.clone(),
            components: Vec::new(),
            entity_proto: proto_entity.clone(),
            children: init_entities(proto_entity.children.clone(), scripting),
            component_flags: ComponentFlags::EMPTY,
            transform: Rc::new(RefCell::new(Transform::Transform2D {
                position: (0.0, 0.0),
                rotation: 0.0,
                scale: (1.0, 1.0),
                r#static: false
            })),
            is_new: true
        };
        
        for proto_component in runtime_entity.entity_proto.components.clone() {
            let flags = scripting.get_component_flags(proto_component.name.as_str());
            runtime_entity.component_flags |= flags;
            let runtime_component = RuntimeComponent {
                data: ScriptingData::from_component_proto(proto_component.clone(), scripting).unwrap(),
                component_proto: proto_component,
                flags,
            };
            runtime_entity.components.push(runtime_component);
        }
        runtime_entities.push(runtime_entity);
    }
    runtime_entities
}
