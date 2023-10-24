pub mod rune;
use std::fmt;

pub type Result<T> = std::result::Result<T, ScriptingError>;

#[derive(Debug, Clone)]
pub struct ScriptingError {
    message: String,
}

#[derive(Debug, Clone)]
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
}

