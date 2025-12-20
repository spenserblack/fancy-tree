//! Module for the state builder.
use super::State;
use crate::git::Git;
use crate::lua::api;
use mlua::Lua;

/// Builds the Lua state.
#[derive(Default)]
pub struct Builder<'git> {
    git: Option<&'git Git>,
}

impl<'git> Builder<'git> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self { git: None }
    }

    /// Adds git to the builder.
    #[must_use]
    pub fn with_git(self, git: &'git Git) -> Self {
        Self { git: Some(git) }
    }

    /// Builds the Lua state.
    pub fn build(self) -> mlua::Result<State<'git>> {
        use mlua::{LuaOptions, StdLib};

        /// The global name of the API.
        const API_NAME: &str = "fancytree";

        let inner = Lua::new_with(StdLib::TABLE | StdLib::STRING, LuaOptions::default())?;

        let api = api::Builder::new().with_path().build(&inner)?;

        if self.git.is_some() {
            // NOTE We don't actually add any utilities here, because we need scoping.
            let git = inner.create_table()?;
            api.set("git", git)?;
        }

        let globals = inner.globals();
        globals.set(API_NAME, api)?;

        let state = State {
            inner,
            git: self.git,
        };
        Ok(state)
    }
}
