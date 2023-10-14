use mlua::{Lua, Result, Table, Value};
use std::collections::HashMap;
use std::mem;

pub struct ScriptingInstance{
    lua: Lua,
    environments: HashMap<String, Table<'static>>,
}

impl ScriptingInstance {
    pub fn new() -> Result<Self> {
        Ok(Self {
            lua: Lua::new(),
            environments: HashMap::new(),
        })
    }

    pub fn load_script(&mut self, env_name: &str, script: &str) -> Result<()> {
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
                return Err(e);
            }
        }

        // Store the environment for future use.
        // Transmute the lifetime of the environment table to 'static.
        // UNSAFE: Ensure that Lua always outlives the use of the table to prevent use-after-free.
        let environment_static: Table<'static> = unsafe { mem::transmute(environment) };
        self.environments.insert(env_name.to_string(), environment_static);

        Ok(())
    }

    pub fn execute_in_environment(&mut self, env_name: &str, script: &str) -> Result<()> {
        if let Some(environment) = self.environments.get(env_name) {
           // Execute the script in the existing environment.
            let loaded_chunk = self.lua.load(script).set_environment(environment.clone());
            loaded_chunk.exec()?;
            Ok(())
        } else {
            Err(mlua::Error::RuntimeError("Environment not found".to_string()))
        }
    }
}
