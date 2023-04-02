use std::path::Path;

use ftldat::error::PackageReadError;
use ftldat::{Package, PackageEntry};
use mlua::{Lua, UserDataMethods};
use mlua::prelude::{LuaResult, LuaTable, LuaUserData};
use crate::lua_error::external_lua_error;

/// Build the module's exports table, governing what is exposed to Lua.
pub fn init(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set("read_package", lua.create_function(read)?)?;
    exports.set("new_package", lua.create_function(new)?)?;

    Ok(exports)
}

//region <Exported adapter functions>
fn new(_: &Lua, (): ()) -> LuaResult<LuaPackageWrapper> {
    Ok(LuaPackageWrapper::new())
}

fn read(_: &Lua, (path, ): (String, )) -> LuaResult<LuaPackageWrapper> {
    LuaPackageWrapper::read_from_path(&path)
        .map_err(external_lua_error)
}
//endregion

struct LuaPackageWrapper {
    package: Package,
}

impl LuaPackageWrapper {
    fn new() -> LuaPackageWrapper {
        LuaPackageWrapper {
            package: Package::new()
        }
    }

    fn read_from_path<P: AsRef<Path>>(path: P) -> Result<LuaPackageWrapper, PackageReadError> {
        ftldat::dat::read_from_path(path)
            .map(|package| {
                LuaPackageWrapper { package }
            })
    }
}

impl LuaUserData for LuaPackageWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("to_file", |_, this, (path, ): (String, )| {
            ftldat::dat::write_package_to_path(&this.package, &path)
                .map_err(external_lua_error)
        });

        methods.add_method_mut("add_entry_from_string", |_, this, (path, content): (String, String)| {
            this.package.add_entry(PackageEntry::from_string(path, content))
                .map_err(external_lua_error)
        });

        methods.add_method_mut("add_entry_from_byte_array", |_, this, (path, content): (String, Vec<u8>)| {
            this.package.add_entry(PackageEntry::from_byte_array(path, content))
                .map_err(external_lua_error)
        });

        methods.add_method_mut("add_entry_from_file", |_, this, (path, source_path): (String, String)| {
            this.package.add_entry(PackageEntry::from_file(path, source_path))
                .map_err(external_lua_error)
        });

        methods.add_method_mut("put_entry_from_string", |_, this, (path, content): (String, String)| {
            this.package.put_entry(PackageEntry::from_string(path, content));
            Ok(())
        });

        methods.add_method_mut("put_entry_from_byte_array", |_, this, (path, content): (String, Vec<u8>)| {
            this.package.put_entry(PackageEntry::from_byte_array(path, content));
            Ok(())
        });

        methods.add_method_mut("put_entry_from_file", |_, this, (path, source_path): (String, String)| {
            this.package.put_entry(PackageEntry::from_file(path, source_path));
            Ok(())
        });

        methods.add_method("read_content_as_string", |_, this, (path, ): (String, )| {
            let maybe_bytes = this.package.content_by_path(path);
            match maybe_bytes {
                None => Ok(None),
                Some(bytes) => {
                    let content = String::from_utf8(bytes)
                        .map_err(external_lua_error)?;
                    Ok(Some(content))
                }
            }
        });

        methods.add_method("read_content_as_byte_array", |_, this, (path, ): (String, )| {
            let maybe_bytes = this.package.content_by_path(path);
            Ok(maybe_bytes)
        });

        methods.add_method_mut("remove", |_, this, (path, ): (String, )| {
            Ok(this.package.remove_entry(path))
        });

        methods.add_method("exists", |_, this, (path, ): (String, )| {
            Ok(this.package.entry_exists(&path))
        });

        methods.add_method_mut("clear", |_, this, ()| {
            Ok(this.package.clear())
        });

        methods.add_method("inner_paths", |_lua, this, ()| {
            Ok(this.package.inner_paths())
        });

        methods.add_method("len", |_, this, ()| {
            Ok(this.package.entry_count())
        });

        methods.add_method("entry_count", |_, this, ()| {
            Ok(this.package.entry_count())
        });

        methods.add_method("extract", |_, this, (path, ): (String, )| {
            this.package.extract(path)
                .map_err(external_lua_error)
        });
    }
}
