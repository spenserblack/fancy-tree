//! Module for creating a Lua state object for the application.
use super::api;
pub use builder::Builder;
use mlua::Lua;

mod builder;

/// Container for the Lua state.
///
/// This helps ensure proper lifetimes for any type that the state uses.
pub struct State {
    /// The actual Lua state.
    inner: Lua,
}

impl State {
    /// The inner Lua state.
    pub fn to_inner(&self) -> &Lua {
        &self.inner
    }
}
