pub mod lua;
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
    fn load_script(&mut self, env_name: &str, script: &str) -> Result<()>;
    fn execute_in_environment(&mut self, env_name: &str, script: &str) -> Result<()>;
}

