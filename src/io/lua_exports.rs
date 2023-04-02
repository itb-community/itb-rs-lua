use std::error::Error;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use mlua::{Lua, UserDataMethods, Variadic};
use mlua::prelude::{LuaError, LuaResult, LuaTable, LuaUserData};
use path_absolutize::Absolutize;

use crate::io::directory::Directory;
use crate::io::file::File;
use crate::io::path_filter::PathFilter;

/// Build the module's exports table, governing what is exposed to Lua.
pub fn init(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set("file", lua.create_function(lua_file)?)?;
    exports.set("directory", lua.create_function(lua_directory)?)?;
    exports.set("save_data_directory", lua.create_function(save_data_directory)?)?;

    Ok(exports)
}

//region <Exported adapter functions>
fn lua_file(_: &Lua, (path, ): (String, )) -> LuaResult<File> {
    let path = normalize(PathBuf::from(path));
    let normalized_path = path.absolutize()
        .map_err(external_lua_error)?;

    file(normalized_path)
        .map_err(external_lua_error)
}

fn lua_directory(_: &Lua, (path, ): (String, )) -> LuaResult<Directory> {
    let path = normalize(PathBuf::from(path));
    let normalized_path = path.absolutize()
        .map_err(external_lua_error)?;

    directory(normalized_path)
        .map_err(external_lua_error)
}

fn save_data_directory(_: &Lua, (): ()) -> LuaResult<Directory> {
    PathFilter::save_data_directory()
        .map(Directory::from)
        .map_err(external_lua_error)
}
//endregion

fn external_lua_error<T: Error + Send + Sync + 'static>(error: T) -> LuaError {
    LuaError::ExternalError(Arc::new(error))
}

fn file<P: AsRef<Path>>(path: P) -> std::io::Result<File> where PathBuf: From<P> {
    if PathFilter::is_whitelisted(&path)? {
        Ok(File::from(path))
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Path points to a file not within an allowed directory"))
    }
}

fn directory<P: AsRef<Path>>(path: P) -> std::io::Result<Directory> where PathBuf: From<P> {
    if PathFilter::is_whitelisted(&path)? {
        Ok(Directory::from(path))
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Path does not point to an allowed directory"))
    }
}

fn normalize(path: PathBuf) -> PathBuf {
    let first_component = path.components().into_iter().next().unwrap();
    match first_component {
        // Rust's path library doesn't implicitly treat 'naked' paths in the form
        // of 'some_file.txt' as relative to current directory, so append . in front
        // to fix this.
        Component::Normal(_) => PathBuf::from(".").join(path),
        // All other cases are anchored in a way that the library can make sense of
        // them, so no alteration is necessary
        _ => path
    }
}

impl LuaUserData for File {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("path", |_, this, ()| {
            Ok(this.path())
        });

        methods.add_method("relative_path", |_, this, ()| {
            this.relative_path()
                .map_err(external_lua_error)
        });

        methods.add_method("name", |_, this, ()| {
            Ok(this.name())
        });

        methods.add_method("name_without_extension", |_, this, ()| {
            Ok(this.name_without_extension())
        });

        methods.add_method("extension", |_, this, ()| {
            Ok(this.extension())
        });

        methods.add_method("parent", |_, this, ()| {
            this.parent()
                .map_err(external_lua_error)
        });

        methods.add_method("root", |_, this, ()| {
            this.root()
                .map_err(external_lua_error)
        });

        methods.add_method("read_to_string", |_, this, ()| {
            this.read_to_string()
                .map_err(external_lua_error)
        });

        methods.add_method("read_to_byte_array", |_, this, ()| {
            this.read_to_byte_array()
                .map_err(external_lua_error)
        });

        methods.add_method("write_string", |_, this, (content, ): (String, )| {
            this.write_string(content)
                .map_err(external_lua_error)
        });

        methods.add_method("append_string", |_, this, (content, ): (String, )| {
            this.append_string(content)
                .map_err(external_lua_error)
        });

        methods.add_method("write_byte_array", |_, this, (content, ): (Vec<u8>, )| {
            this.write_byte_array(content)
                .map_err(external_lua_error)
        });

        methods.add_method("copy", |_, this, (destination, ): (String, )| {
            let path = normalize(PathBuf::from(destination));
            let normalized_path = path.absolutize()
                .map_err(external_lua_error)?;

            Ok(this.copy(&normalized_path).map_err(external_lua_error)?)
        });

        methods.add_method("move", |_, this, (destination, ): (String, )| {
            let path = normalize(PathBuf::from(destination));
            let normalized_path = path.absolutize()
                .map_err(external_lua_error)?;

            Ok(this.move_file(&normalized_path).map_err(external_lua_error)?)
        });

        methods.add_method("exists", |_, this, ()| {
            Ok(this.exists())
        });

        methods.add_method("delete", |_, this, ()| {
            this.delete()
                .map_err(external_lua_error)
        });
    }
}

impl LuaUserData for Directory {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("path", |_, this, ()| {
            Ok(this.path())
        });

        methods.add_method("relative_path", |_, this, ()| {
            this.relative_path()
                .map_err(external_lua_error)
        });

        methods.add_method("name", |_, this, ()| {
            Ok(this.name())
        });

        methods.add_method("parent", |_, this, ()| {
            this.parent()
                .map_err(external_lua_error)
        });

        methods.add_method("root", |_, this, ()| {
            this.root()
                .map_err(external_lua_error)
        });

        methods.add_method("relativize", |_, this, (path,): (String,)| {
            Ok(this.relativize(path))
        });

        methods.add_method("file", |_, this, (paths,): (Variadic<String>,)| {
            let path: PathBuf = paths.iter().collect();
            let path = this.path.join(path);
            let normalized_path = path.absolutize()
                .map_err(external_lua_error)?;

            file(normalized_path)
                .map_err(external_lua_error)
        });

        methods.add_method("directory", |_, this, (paths,): (Variadic<String>,)| {
            let path: PathBuf = paths.iter().collect();
            let path = this.path.join(path);
            let normalized_path = path.absolutize()
                .map_err(external_lua_error)?;

            directory(normalized_path)
                .map_err(external_lua_error)
        });

        methods.add_method("files", |_, this, ()| {
            this.files()
                .map_err(external_lua_error)
        });

        methods.add_method("directories", |_, this, ()| {
            this.directories()
                .map_err(external_lua_error)
        });

        methods.add_method("make_directories", |_, this, ()| {
            this.make_directories()
                .map_err(external_lua_error)
        });

        methods.add_method("exists", |_, this, ()| {
            Ok(this.exists())
        });

        methods.add_method("is_ancestor", |_, this, (path,): (String,)| {
            this.is_ancestor(path)
                .map_err(external_lua_error)
        });

        methods.add_method("delete", |_, this, ()| {
            this.delete()
                .map_err(external_lua_error)
        });
    }
}
