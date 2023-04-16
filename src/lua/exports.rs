use mlua::Lua;
use mlua::prelude::{LuaResult, LuaTable};

use crate::lua;

/// Build the module's exports table, governing what is exposed to Lua.
pub fn init(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set("io", lua::io::init(lua)?)?;
    exports.set("ftldat", lua::ftldat::init(lua)?)?;

    Ok(exports)
}
