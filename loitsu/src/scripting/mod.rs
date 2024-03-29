pub mod rune_runtime;
use crate::ecs::{ComponentFlags, RuntimeTransform, ECS};
use crate::input::InputState;
use crate::scene_management::Property;
use crate::{
    rendering::drawable::{DrawableProperty, DrawablePrototype},
    scene_management::Component,
};
use bitcode;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

pub type Result<T> = std::result::Result<T, ScriptingError>;

#[derive(Debug, Clone)]
pub struct ScriptingError {
    message: String,
}

#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct ScriptingSource {
    pub name: String,
    pub source: String,
}

impl ScriptingError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ScriptingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub enum EntityUpdate {
    AddDrawable(DrawablePrototype),
    RemoveDrawable(u32),
    SetDrawableProperty(u32, String, DrawableProperty),
}

pub trait ScriptingInstance: Sized {
    type Data: ScriptingData<Self>;
    fn new_with_sources(sources: Vec<ScriptingSource>, ecs: Arc<RwLock<ECS<Self>>>) -> Result<Self>
    where
        Self: Sized;
    fn new_uninitialized() -> Result<Self>
    where
        Self: Sized;
    fn initialize(
        &mut self,
        sources: Vec<ScriptingSource>,
        input_state: Arc<Mutex<InputState>>,
        ecs: Arc<RwLock<ECS<Self>>>,
    ) -> Result<()>;
    fn call<T>(&mut self, path: [&str; 2], args: T) -> Result<rune::runtime::Value>
    where
        T: rune::runtime::Args;
    fn run_component_methods<T>(
        &mut self,
        entities: &[Arc<Mutex<crate::ecs::RuntimeEntity<Self>>>],
        lookup: HashMap<u32, Arc<Mutex<crate::ecs::RuntimeEntity<Self>>>>,
        method: ComponentFlags,
    ) -> Vec<(Arc<Mutex<RuntimeTransform>>, Vec<EntityUpdate>)>;
    fn get_component_flags(&self, component_name: &str) -> ComponentFlags;
}

pub trait ScriptingData<T>
where
    T: ScriptingInstance,
{
    fn from_component_proto(proto: Component, instance: &mut T) -> Result<Self>
    where
        Self: Sized;
    fn to_component_proto(&self, proto: &Component) -> Result<Component>;
    fn set_property(&mut self, name: &str, value: Property) -> Result<()>;
}
