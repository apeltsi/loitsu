pub mod rune_runtime;
use std::fmt;
use crate::scene_management::Component;
use bitcode;

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


pub trait ScriptingInstance {
    fn new_with_sources(sources: Vec<ScriptingSource>) -> Result<Self> where Self: Sized;
    fn call<T>(&mut self, path: [&str; 2], args: T) -> Result<rune::runtime::Value> where T: rune::runtime::Args;
}

pub trait ScriptingData {
    // This function taking in a rune instance is less than ideal but ill try to deal with it in
    // the future :)
    fn from_component_proto(proto: Component, instance: &mut rune_runtime::RuneInstance) -> Result<Self> where Self: Sized;
    fn to_component_proto(&self, proto: &Component) -> Result<Component>;
}
