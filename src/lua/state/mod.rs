//! Module for creating a Lua state object for the application.
use crate::git::Git;
pub use builder::Builder;
use mlua::Lua;
use std::ffi::OsString;

mod builder;

/// Container for the Lua state.
///
/// This helps ensure proper lifetimes for any type that the state uses.
pub struct State<'git> {
    /// The actual Lua state.
    inner: Lua,
    /// An optional git state for interfacing with a repository.
    git: Option<&'git Git>,
}

impl<'git> State<'git> {
    /// The inner Lua state.
    pub fn to_inner(&self) -> &Lua {
        &self.inner
    }

    /// Gets the contained git instance.
    pub fn git(&self) -> Option<&'git Git> {
        self.git
    }

    /// Runs the function in a scope where git utilities are potentially available.
    pub fn in_git_scope<T, F>(&self, f: F) -> mlua::Result<T>
    where
        F: FnOnce() -> mlua::Result<T>,
    {
        // HACK We can't build out the git API statically (like we can with the path
        // API) because of lifetimes.
        // HACK Both git and git API must exist, so we can use a shortcut if neither exist.
        let Some(git) = self.git else { return f() };
        let Some(git_api) = self.git_api()? else {
            return f();
        };

        self.inner.scope(|scope| {
            let is_ignored = scope.create_function(|_lua, path: OsString| {
                let is_ignored = git.is_ignored(path).unwrap_or(false);
                Ok(is_ignored)
            })?;
            git_api.set("is_ignored", is_ignored)?;
            f()
        })
    }

    /// Gets a reference to the git table.
    fn git_api(&self) -> mlua::Result<Option<mlua::Table>> {
        let globals = self.inner.globals();
        // TODO These hard-coded keys should be shared variables instead.
        let api = globals.get::<mlua::Table>("fancytree")?;
        api.get::<Option<mlua::Table>>("git")
    }
}
