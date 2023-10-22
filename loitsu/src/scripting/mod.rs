pub mod rune;
use std::fmt;

pub type Result<T> = std::result::Result<T, ScriptingError>;

#[derive(Debug, Clone)]
pub struct ScriptingError {
    message: String,
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
    fn new() -> Result<Self> where Self: Sized;
    fn add_script(&mut self, name: &str, path: &str, script: &str) -> Result<()>;
}

