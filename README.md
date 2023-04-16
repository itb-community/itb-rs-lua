# ITB Rust Lua bridge

Rust-Lua bridge that exports several useful functions, to aid in development of mods for the game Into the Breach.

# Building

This section assumes you have Rust set up with MSVC. If not, see here: https://www.rust-lang.org/learn/get-started.

Building for release mode with MINGW should also be possible, and potentially a bit simpler, but I didn't want to try
getting MINGW set up yet.

### Development

For development, the build process is very simple:

1. Open a terminal in the project's root directory.
2. Run `cargo build`.

### Release

For release (as in, getting a .dll that Lua can interface with), the build process is quite a bit more involved.

1. Change configuration to build the library in module mode.
    1. Go to `Cargo.toml`.
    2. Find the `[dependencies]` section.
    3. Find the entry for `mlua` and replace `"vendored"` with `"module"`.
        - Or just comment/uncomment the prepared entries.
    - For explanation why this is needed, see the [Troubleshooting](#troubleshooting) section below.
2. Open a terminal in the project's root directory.
3. (First time only) Add `i686-pc-windows-msvc` target with the command `rustup target add i686-pc-windows-msvc`.
4. Specify environment variables:
    - `LUA_INC=lua/include` - path to Lua headers
    - `LUA_LIB=lua/lua5.1` - path to Lua .lib file
    - `LUA_LIB_NAME=lua/lua5.1` - same path as in `LUA_LIB`
5. Run `cargo build --lib --release --target=i686-pc-windows-msvc`

Steps 4 and 5 are automated in the form of `build.sh` script.

Compiled .dll will be available in `./target/i686-pc-windows-msvc/release/itb_rs_lua.dll`.

# Usage

Load the library in your Lua script:

```lua
-- load the dll - this exposes `itb_rs_lua` global variable,
-- with functions `new_package` and `read_package`
package.loadlib("itb_rs_lua.dll", "luaopen_itb_rs_lua")()

-- access exported fields or functions
local ftldat_module = itb_rs_lua.ftldat;
local io_module = itb_rs_lua.io;
```

# Troubleshooting

The build process for getting a .dll that can interface with Lua is a little finicky.

Into the Breach runs as a 32-bit application, so the library has to be compiled with 32-bit target.

Also, the library has to be built in `mlua`'s [module mode](https://github.com/khvzak/mlua#module-mode), otherwise the
game crashes during exit. The crash doesn't *actually* cause any issues, as far as I could tell, but it does leave sort
of a sour aftertaste after getting everything else to work. It also has the advantage of producing a smaller binary.

Building in module mode under Windows requires linking to a Lua dll (as mentioned in the link).
This is what the `lua` directory and `build.sh` script are for - if you don't want to run the script file, you'll need
to set the variables from the script in your desired environment.

# Attributions

Project setup, as well as linking the compiled .dll file and loading it in Lua was based off of https://github.com/voidshine/renoise_tools. 
