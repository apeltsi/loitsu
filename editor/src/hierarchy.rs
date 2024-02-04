use loitsu::{
    ecs::{RuntimeEntity, ECS},
    scripting::ScriptingInstance,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HierarchyElement {
    pub name: String,
    pub id: String,
    pub children: Vec<HierarchyElement>,
}

pub fn generate_hierarchy<T>(ecs: &ECS<T>) -> Vec<HierarchyElement>
where
    T: ScriptingInstance,
{
    let mut hierarchy = Vec::new();
    // lets walk the ecs tree and generate the hierarchy
    for entity in ecs.get_runtime_entities() {
        hierarchy.push(generate_hierarchy_element(&entity.lock().unwrap()));
    }
    hierarchy
}

fn generate_hierarchy_element<T>(entity: &RuntimeEntity<T>) -> HierarchyElement
where
    T: ScriptingInstance,
{
    let mut children = Vec::new();
    for child in &entity.children {
        children.push(generate_hierarchy_element(&child.lock().unwrap()));
    }
    HierarchyElement {
        name: entity.get_name().to_string(),
        id: entity.get_id().to_string(),
        children,
    }
}
