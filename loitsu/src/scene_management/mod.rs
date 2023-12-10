#[cfg(feature = "scene_generation")]
use serde_json::{Value, Map};

use std::collections::HashMap;
use bitcode;

use crate::ecs::Transform;

#[cfg_attr(feature = "scene_generation", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
#[bitcode(recursive)]
pub enum Property {
    String(String),
    Number(f32),
    Boolean(bool),
    Array(Vec<Property>),
    EntityReference(String), // Reference to another entity in the scene (Represents the ID)
    ComponentReference(String), // Reference to another component in the scene (Represents the ID)
}

#[cfg_attr(feature = "scene_generation", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<Entity>,
    pub required_assets: Vec<String>,
    pub shards: Vec<String>
}

#[cfg_attr(feature = "scene_generation", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
#[bitcode(recursive)]
pub struct Entity {
    pub name: String,
    #[bitcode_hint(ascii_lowercase)]
    pub id: String,
    pub components: Vec<Component>,
    pub children: Vec<Entity>,
    pub transform: Transform,
}

#[cfg_attr(feature = "scene_generation", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct Component {
    pub name: String,
    #[bitcode_hint(ascii_lowercase)]
    pub id: String,
    pub properties: HashMap<String, Property>,
}

impl Scene {
    pub fn new(name: String) -> Scene {
        Scene {
            name,
            entities: Vec::new(),
            required_assets: Vec::new(),
            shards: Vec::new()
        }
    }
    #[cfg(feature = "scene_generation")]
    pub fn from_json(name: String, json: String) -> Scene {
        let v: Value = serde_json::from_str(&json).unwrap();
        let name = name;
        let mut scene = Scene::new(name);
        scene.entities = collect_entities(v["entities"].as_array().unwrap().to_vec());
        scene
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    #[cfg(feature = "scene_generation")]
    pub fn to_json(&self) -> String {
        let mut entities = Vec::new();
        for entity in self.entities.clone() {
            entities.push(entity.to_json());
        }
        let mut scene: HashMap<&str, Value> = HashMap::new();
        scene.insert("name", serde_json::Value::String(self.name.clone()));
        scene.insert("entities", serde_json::Value::Array(entities));
        scene.insert("required_assets", serde_json::Value::Array(self.required_assets.clone().into_iter().map(|asset| serde_json::Value::String(asset)).collect()));
        serde_json::to_string(&scene).unwrap()
    }
}
#[cfg(feature = "scene_generation")]
fn collect_entities(entities: Vec<Value>) -> Vec<Entity> {
    // lets iterate over the entities, collect their components, properties AND children
    // recursively
    let mut out_entities = Vec::new();
    for entity in entities {
        let name = entity["name"].as_str().unwrap().to_string();
        let id = entity["id"].as_str().unwrap().to_string();
        let mut out_entity = Entity::new(name, id);
        let components = entity["components"].as_array().unwrap();
        for component in components {
            let name = component["name"].as_str().unwrap().to_string();
            let id = component["id"].as_str().unwrap().to_string();
            let mut out_component = Component::new(name, id);
            let properties = component["properties"].as_object().unwrap();
            for property in properties {
                let property_name = property.0.clone();
                out_component.add_property(property_name, json_value_as_property(property.1.clone()));
            }
            out_entity.add_component(out_component);
        }
        out_entity.children = collect_entities(entity["children"].as_array().unwrap().to_vec());
        
        // lets parse the transform
        let transform = entity["transform"].as_object().unwrap();
        out_entity.transform = Transform::from_json(transform);
        out_entities.push(out_entity);
    }
    out_entities
}


impl Entity {
    pub fn new(name: String, id: String) -> Entity {
        Entity {
            name,
            id,
            components: Vec::new(),
            children: Vec::new(),
            transform: Transform::Transform2D {
                position: (0.0, 0.0),
                rotation: 0.0,
                scale: (1.0, 1.0),
                r#static: false,
                has_changed: true,
                changed_frame: 0
            }
        }
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }

    #[cfg(feature = "scene_generation")]
    pub fn to_json(&self) -> Value {
        let mut components = Vec::new();
        for component in self.components.clone() {
            components.push(component.to_json());
        }
        let mut entity = Map::new();
        entity.insert("name".to_string(), serde_json::Value::String(self.name.clone()));
        entity.insert("id".to_string(), serde_json::Value::String(self.id.clone()));
        entity.insert("components".to_string(), serde_json::Value::Array(components));
        entity.insert("transform".to_string(), self.transform.clone().to_json());
        let mut children = Vec::new();
        for child in self.children.clone() {
            children.push(child.to_json());
        }
        entity.insert("children".to_string(), serde_json::Value::Array(children));
        Value::Object(entity)
    }
}

impl Component {
    pub fn new(name: String, id: String) -> Component {
        Component {
            name,
            id,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, name: String, property: Property) {
        self.properties.insert(name, property);
    }
    #[cfg(feature = "scene_generation")]
    pub fn to_json(&self) -> Value {
        let mut properties = Map::new();
        for property in self.properties.clone() {
            properties.insert(property.0, property.1.to_json());
        }
        let mut component = Map::new();
        component.insert("name".to_string(), serde_json::Value::String(self.name.clone()));
        component.insert("properties".to_string(), serde_json::Value::Object(properties));
        Value::Object(component)
    }
}

#[cfg(feature = "scene_generation")]
fn json_value_as_property(value: Value) -> Property {
    match value {
        Value::String(s) => Property::String(s),
        Value::Number(n) => Property::Number(n.as_f64().unwrap() as f32),
        Value::Bool(b) => Property::Boolean(b),
        Value::Array(a) => {
            let mut out = Vec::new();
            for v in a {
                out.push(json_value_as_property(v));
            }
            Property::Array(out)
        }
        _ => panic!("Unsupported property type!"),
    }
}

#[cfg(feature = "scene_generation")]
impl Property {
    pub fn to_json(&self) -> Value {
        match self {
            Property::String(s) => Value::String(s.clone()),
            Property::Number(n) => Value::Number(serde_json::Number::from_f64(*n as f64).unwrap()),
            Property::Boolean(b) => Value::Bool(*b),
            Property::Array(a) => {
                let mut out = Vec::new();
                for v in a {
                    out.push(v.to_json());
                }
                Value::Array(out)
            }
            _ => panic!("Unsupported property type!"),
        }
    }
}
