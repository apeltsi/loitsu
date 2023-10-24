pub struct Scene {
    pub name: String,
    pub entities: Vec<Entity>,
}

pub struct Entity {
    pub name: String,
    pub id: String,
    pub components: Vec<Component>,
}

pub struct Component {
    pub name: String,
    pub properties: Vec<String>,
}
