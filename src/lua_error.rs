use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use mlua::prelude::LuaError;

/// Some errors include cause wrapped in a Box, which prevents automatic impl of Send and Sync
/// traits, which makes these errors unsafe to share between threads.
/// To fix this, just convert the errors to a string - we still get the information we need for
/// traceability and debugging purposes.
#[derive(Debug)]
struct LuaErrorWrapper {
    message: String,
}

impl Display for LuaErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for LuaErrorWrapper {}

pub fn external_lua_error<T: Error>(error: T) -> LuaError {
    let wrapped_error = LuaErrorWrapper {
        message: error.to_string()
    };
    LuaError::ExternalError(Arc::new(wrapped_error))
}
