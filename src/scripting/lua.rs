use mlua::{Lua, Table, prelude::LuaError};
use crate::scripting::ScriptingError;
use std::collections::HashMap;
use std::mem;
use crate::ScriptingInstance;

pub type Result<T> = std::result::Result<T, ScriptingError>;

pub struct LuaInstance {
    lua: Lua,
    environments: HashMap<String, Table<'static>>,
}

impl From<LuaError> for ScriptingError {
    fn from(error: LuaError) -> Self {
        ScriptingError {
            message: format!("{}", error),
        }
    }
}

impl ScriptingInstance for LuaInstance {
    fn new() -> Result<Self> {
        Ok(Self {
            lua: Lua::new(),
            environments: HashMap::new(),
        })
    }

    fn load_script(&mut self, env_name: &str, script: &str) -> Result<()> {
        // Create a new environment.
        let environment = self.lua.create_table()?;

        let print_prefix = format!("[{}]: ", env_name);
        // Lets create a print function
        let print = self.lua.create_function(move |_, s: String| {
            println!("{}{}", print_prefix, s);
            Ok(())
        })?;

        environment.set("print", print)?;

        // Execute the script in the new environment.
        let loaded_chunk = self.lua.load(script).set_environment(environment.clone());
        match loaded_chunk.exec() {
            Ok(_) => {}
            Err(e) => {
                println!("Error when loading script'{}': {}", env_name,e);
                return Err(ScriptingError::new(e.to_string().as_str()));
            }
        }

        // Store the environment for future use.
        // Transmute the lifetime of the environment table to 'static.
        let environment_static: Table<'static> = unsafe { mem::transmute(environment) };
        self.environments.insert(env_name.to_string(), environment_static);

        Ok(())
    }

    fn execute_in_environment(&mut self, env_name: &str, script: &str) -> Result<()> {
        if let Some(environment) = self.environments.get(env_name) {
            // Execute the script in the existing environment.
            let loaded_chunk = self.lua.load(script).set_environment(environment.clone());
            loaded_chunk.exec()?;
            Ok(())
        } else {
            Err(ScriptingError::new("Environment not found"))
        }
    }
}
