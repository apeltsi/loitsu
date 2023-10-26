use crate::scene_management::{Scene, Entity, Component};
use crate::scripting::{ScriptingData, ScriptingInstance};
use crate::scripting::rune_runtime::{RuneInstance, RuneComponent};

pub struct ECS {
    pub active_scene: Scene,
    pub static_scene: Option<Scene>,
    runtime_entities: Vec<RuntimeEntity>,
}

struct RuntimeEntity {
    name: String,
    id: String,
    components: Vec<RuntimeComponent>,
    entity_proto: Entity,
    children: Vec<RuntimeEntity>,
}

struct RuntimeComponent {
    name: String,
    data: RuneComponent,
    component_proto: Component,
}

impl ECS {
    pub fn new() -> ECS {
        ECS {
            active_scene: Scene::new("INITIAL_SCENE".to_string()),
            static_scene: None,
            runtime_entities: Vec::new(),
        }
    }

    pub fn load_scene(&mut self, scene: Scene, scripting: &mut RuneInstance) {
        self.active_scene = scene.clone();
        self.runtime_entities = init_entities(scene.entities, scripting);
    }

    pub fn run_build_step(&mut self, scripting: &mut RuneInstance) {
        // Lets iterate over the entities and run the build step on each component
        for runtime_entity in &self.runtime_entities {
            for runtime_component in &runtime_entity.components {
                match &runtime_component.data.data {
                    Some(data) => {
                        scripting.call([runtime_component.name.as_str(), "build"], (data.clone(), )).unwrap();
                    },
                    None => {
                        scripting.call([runtime_component.name.as_str(), "build"], (rune::runtime::Value::EmptyTuple, )).unwrap();
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.active_scene = Scene::new("INITIAL_SCENE".to_string());
        self.static_scene = None;
        self.runtime_entities = Vec::new();
    }

    #[cfg(feature = "scene_generation")]
    pub fn as_scene(&self, scripting: &mut RuneInstance) -> Scene {
        Scene {
            name: self.active_scene.name.clone(),
            entities: self.runtime_entities.iter().map(|runtime_entity| runtime_entity.as_entity(scripting)).collect(),
            required_assets: Vec::new()
        }
    }
}

impl RuntimeEntity {
    #[cfg(feature = "scene_generation")]
    pub fn as_entity(&self, scripting: &mut RuneInstance) -> Entity {
        Entity {
            name: self.name.clone(),
            id: self.id.clone(),
            components: self.components.iter().map(|runtime_component| runtime_component.as_component(scripting)).collect(),
            children: self.children.iter().map(|runtime_entity| runtime_entity.as_entity(scripting)).collect(),
        }
    }
}

impl RuntimeComponent {
    #[cfg(feature = "scene_generation")]
    pub fn as_component(&self, scripting: &mut RuneInstance) -> Component {
        self.data.to_component_proto(&self.component_proto, scripting).unwrap()
    }
}

fn init_entities(proto_entities: Vec<Entity>, scripting: &mut RuneInstance) -> Vec<RuntimeEntity> {
    // Lets recursively iterate over the entities and create a runtime entity for each one
    let mut runtime_entities = Vec::new();
    for proto_entity in proto_entities {
        let mut runtime_entity = RuntimeEntity {
            name: proto_entity.name.clone(),
            id: proto_entity.id.clone(),
            components: Vec::new(),
            entity_proto: proto_entity.clone(),
            children: init_entities(proto_entity.children.clone(), scripting),
        };
        for proto_component in runtime_entity.entity_proto.components.clone() {
            let runtime_component = RuntimeComponent {
                name: proto_component.name.clone(),
                data: RuneComponent::from_component_proto(proto_component.clone(), scripting).unwrap(),
                component_proto: proto_component,
            };
            runtime_entity.components.push(runtime_component);
        }
        runtime_entities.push(runtime_entity);
    }
    runtime_entities
}
