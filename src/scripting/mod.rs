use mlua::prelude::*;


pub fn init_scripting() -> LuaResult<Lua> {
    let lua = Lua::new();
    
    Ok(lua)
}
