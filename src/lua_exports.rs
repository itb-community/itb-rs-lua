use mlua::Lua;
use mlua::prelude::{LuaResult, LuaTable};

use crate::{io};

/// Build the module's exports table, governing what is exposed to Lua.
pub fn init(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set("io", io::lua_exports::init(lua)?)?;

    Ok(exports)
}