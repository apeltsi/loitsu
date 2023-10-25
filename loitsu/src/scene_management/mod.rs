#[cfg(feature = "scene_generation")]
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Property {
    String(String),
    Number(f32),
    Boolean(bool),
    Array(Vec<Property>),
    EntityReference(String), // Reference to another entity in the scene (Represents the ID)
    ComponentReference(String), // Reference to another component in the scene (Represents the ID)
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub id: String,
    pub components: Vec<Component>,
    pub children: Vec<Entity>,
}
#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub properties: HashMap<String, Property>,
}

impl Scene {
    pub fn new(name: String) -> Scene {
        Scene {
            name,
            entities: Vec::new(),
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
            let mut out_component = Component::new(name);
            let properties = component["properties"].as_object().unwrap();
            for property in properties {
                let property_name = property.0.clone();
                out_component.add_property(property_name, json_value_as_property(property.1.clone()));
            }
            out_entity.add_component(out_component);
        }
        out_entity.children = collect_entities(entity["children"].as_array().unwrap().to_vec());
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
        }
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }
}

impl Component {
    pub fn new(name: String) -> Component {
        Component {
            name,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, name: String, property: Property) {
        self.properties.insert(name, property);
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
