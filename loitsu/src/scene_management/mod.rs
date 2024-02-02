#[cfg(feature = "scene_generation")]
use serde_json::{Map, Value};

use bitcode;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::ecs::Transform;

#[cfg_attr(
    feature = "scene_generation",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
#[bitcode(recursive)]
pub enum Property {
    String(String),
    Number(f32),
    Boolean(bool),
    Array(Vec<Property>),
    EntityReference(u32), // Reference to another entity in the scene (Represents the ID)
    ComponentReference(u32), // Reference to another component in the scene (Represents the ID)
}

impl Debug for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Property::String(s) => write!(f, "{}", s),
            Property::Number(n) => write!(f, "{}", n),
            Property::Boolean(b) => write!(f, "{}", b),
            Property::Array(a) => {
                write!(f, "[")?;
                for (i, item) in a.iter().enumerate() {
                    write!(f, "{:?}", item)?;
                    if i != a.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Property::EntityReference(id) => write!(f, "EntityReference({})", id),
            Property::ComponentReference(id) => write!(f, "ComponentReference({})", id),
        }
    }
}

impl Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Property::String(s) => write!(f, "{}", s),
            Property::Number(n) => write!(f, "{}", n),
            Property::Boolean(b) => write!(f, "{}", b),
            Property::Array(a) => {
                write!(f, "[")?;
                for (i, item) in a.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i != a.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Property::EntityReference(id) => write!(f, "EntityReference({})", id),
            Property::ComponentReference(id) => write!(f, "ComponentReference({})", id),
        }
    }
}

#[cfg_attr(
    feature = "scene_generation",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<Entity>,
    pub required_assets: Vec<String>,
    pub shards: Vec<String>,
    pub id_space: u32,
}

#[cfg_attr(
    feature = "scene_generation",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
#[bitcode(recursive)]
pub struct Entity {
    pub name: String,
    pub id: u32,
    pub components: Vec<Component>,
    pub children: Vec<Entity>,
    pub transform: Transform,
}

#[cfg_attr(
    feature = "scene_generation",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, bitcode::Encode, bitcode::Decode)]
pub struct Component {
    pub name: String,
    pub id: u32,
    pub properties: HashMap<String, Property>,
}

impl Scene {
    pub fn new(name: String) -> Scene {
        Scene {
            name,
            entities: Vec::new(),
            required_assets: Vec::new(),
            shards: Vec::new(),
            id_space: 0,
        }
    }

    #[cfg(feature = "scene_generation")]
    pub fn from_json(name: String, json: String) -> Scene {
        let v: Value = serde_json::from_str(&json).unwrap();
        let name = name;
        let mut scene = Scene::new(name);
        scene.entities = collect_entities(v["entities"].as_array().unwrap().to_vec());
        scene.id_space = v["id_space"].as_number().unwrap().as_u64().unwrap() as u32;
        scene
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    #[cfg(feature = "scene_generation")]
    pub fn to_json(&self) -> String {
        let mut entities = Vec::new();
        let mut scene_entities = self.entities.clone();
        let (scene_entities, id_space) = normalize_ids(&mut scene_entities);
        for entity in scene_entities {
            entities.push(entity.to_json());
        }
        let mut scene: HashMap<&str, Value> = HashMap::new();
        scene.insert("name", serde_json::Value::String(self.name.clone()));
        scene.insert("entities", serde_json::Value::Array(entities));
        scene.insert(
            "required_assets",
            serde_json::Value::Array(
                self.required_assets
                    .clone()
                    .into_iter()
                    .map(|asset| serde_json::Value::String(asset))
                    .collect(),
            ),
        );
        scene.insert(
            "id_space",
            serde_json::Value::Number(serde_json::Number::from(id_space)),
        );
        serde_json::to_string(&scene).unwrap()
    }

    pub fn reserve_ids(&mut self) {
        let id_space_begin = crate::util::id::reserve_id_space(self.id_space);
        for entity in self.entities.iter_mut() {
            entity.reserve_ids(id_space_begin);
        }
    }
}

/// Will change all entities and components to the smallest possible IDs while preserving
/// uniqueness
#[allow(dead_code)]
fn normalize_ids(entities: &mut Vec<Entity>) -> (Vec<Entity>, u32) {
    let mut id_map: HashMap<u32, u32> = HashMap::new();
    let mut next_id = 0;
    for entity in entities.iter_mut() {
        let old_id = entity.id;
        let new_id = match id_map.get(&old_id) {
            Some(id) => *id,
            None => {
                id_map.insert(old_id, next_id);
                next_id += 1;
                next_id - 1
            }
        };
        entity.id = new_id;
        for component in entity.components.iter_mut() {
            let old_id = component.id;
            let new_id = match id_map.get(&old_id) {
                Some(id) => *id,
                None => {
                    id_map.insert(old_id, next_id);
                    next_id += 1;
                    next_id - 1
                }
            };
            component.id = new_id;
        }
        normalize_ids(&mut entity.children);
    }
    (entities.clone(), next_id)
}

#[cfg(feature = "scene_generation")]
fn collect_entities(entities: Vec<Value>) -> Vec<Entity> {
    // lets iterate over the entities, collect their components, properties AND children
    // recursively

    let mut out_entities = Vec::new();
    for entity in entities {
        let name = entity["name"].as_str().unwrap().to_string();
        let id = entity["id"].as_str().unwrap();
        let id = parse_component_or_entity_id(id).unwrap();
        let mut out_entity = Entity::new(name, id);
        let components = entity["components"].as_array().unwrap();
        for component in components {
            let name = component["name"].as_str().unwrap().to_string();
            let id = component["id"].as_str().unwrap();
            let id = parse_component_or_entity_id(id).unwrap();
            let mut out_component = Component::new(name, id);
            let properties = component["properties"].as_object().unwrap();
            for property in properties {
                let property_name = property.0.clone();
                out_component
                    .add_property(property_name, json_value_as_property(property.1.clone()));
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
    pub fn new(name: String, id: u32) -> Entity {
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
            },
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
        entity.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        entity.insert(
            "id".to_string(),
            serde_json::Value::String(format!("EID{}", self.id)),
        );
        entity.insert(
            "components".to_string(),
            serde_json::Value::Array(components),
        );
        entity.insert("transform".to_string(), self.transform.clone().to_json());
        let mut children = Vec::new();
        for child in self.children.clone() {
            children.push(child.to_json());
        }
        entity.insert("children".to_string(), serde_json::Value::Array(children));
        Value::Object(entity)
    }

    pub fn reserve_ids(&mut self, id_space_begin: u32) {
        self.id += id_space_begin;
        for component in self.components.iter_mut() {
            component.id += id_space_begin;
            for property in component.properties.iter_mut() {
                match property.1 {
                    Property::EntityReference(ref mut id) => *id += id_space_begin,
                    Property::ComponentReference(ref mut id) => *id += id_space_begin,
                    _ => {}
                }
            }
        }
        for child in self.children.iter_mut() {
            child.reserve_ids(id_space_begin);
        }
    }
}

impl Component {
    pub fn new(name: String, id: u32) -> Component {
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
        component.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        component.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties),
        );
        component.insert(
            "id".to_string(),
            serde_json::Value::String(format!("CID{}", self.id)),
        );
        Value::Object(component)
    }
}

#[allow(dead_code)]
fn parse_component_or_entity_id(id: &str) -> Option<u32> {
    let id = id[3..].parse::<u32>();
    if let Ok(id) = id {
        return Some(id);
    }
    None
}

#[cfg(feature = "scene_generation")]
fn json_value_as_property(value: Value) -> Property {
    match value {
        Value::String(s) => {
            if s.starts_with("EID") {
                if let Some(id) = parse_component_or_entity_id(&s) {
                    return Property::EntityReference(id);
                }
            } else if s.starts_with("CID") {
                if let Some(id) = parse_component_or_entity_id(&s) {
                    return Property::ComponentReference(id);
                }
            }
            Property::String(s)
        }
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
