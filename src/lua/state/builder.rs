//! Module for the state builder.
use super::State;
use mlua::Lua;
use crate::lua::api;

/// Builds the Lua state.
pub struct Builder;

impl Builder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self
    }

    /// Builds the Lua state.
    #[must_use]
    pub fn build(self) -> mlua::Result<State> {
        use mlua::{LuaOptions, StdLib};

        /// The global name of the API.
        const API_NAME: &str = "fancytree";

        let inner = Lua::new_with(StdLib::TABLE | StdLib::STRING, LuaOptions::default())?;

        let api = api::Builder::new().with_path().build(&inner)?;
        let globals = inner.globals();
        globals.set(API_NAME, api)?;

        let state = State { inner };
        Ok(state)
    }
}
