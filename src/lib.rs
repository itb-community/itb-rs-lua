mod io;
mod lua;

#[no_mangle]
pub extern "C" fn luaopen_itb_rs(lua_state: *mut mlua::lua_State) -> i32 {
    // Leak the Lua purposefully because it's supposed to live for the duration of the program.
    // It should be owned by the game, so as a client DLL, we can assume it's truly 'static.
    let lua = unsafe { mlua::Lua::init_from_ptr(lua_state) }.into_static();

    let export = lua::exports::init(&lua).expect("Failed to initialize module export table");
    lua.globals().set("itb_rs", export).unwrap();

    0
}
