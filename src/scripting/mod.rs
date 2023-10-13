use mlua::prelude::*;

pub struct ScriptingInstance<'lua> {
    lua: Lua,
    environments: Vec<LuaTable<'lua>>,
}

impl<'lua> ScriptingInstance<'lua> {
    pub fn load_script(&'lua mut self, script: &str) -> LuaResult<()> {
        let env = self.lua.create_table()?;
        self.environments.push(env.clone());
        self.lua.load(script).set_environment(env).exec()?;
        Ok(())
    }
}

pub fn init_scripting<'lua>() -> LuaResult<ScriptingInstance<'lua>> {
    let lua = Lua::new();
    
    Ok(ScriptingInstance {
        lua,
        environments: Vec::new(),
    })
}

