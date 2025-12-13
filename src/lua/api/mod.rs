//! Module for creating the `fancytree` API for Lua.
use mlua::Lua;

mod path;

/// Builder for the API table.
pub struct Builder {
    /// Adds `.path` API namespace when true.
    add_path_api: bool,
}

impl Builder {
    /// Creates a new builder.
    #[inline]
    pub fn new() -> Builder {
        Self {
            add_path_api: false,
        }
    }

    /// Instructs the builder to add the `.path` namespace that provides path utilities.
    #[must_use]
    pub fn with_path(self) -> Self {
        Self {
            add_path_api: true,
            ..self
        }
    }

    /// Builds the API table.
    pub fn build(self, lua: &Lua) -> mlua::Result<mlua::Table> {
        let api = Self::core(lua)?;
        let path_api = self.add_path_api.then(|| path::create(lua)).transpose()?;
        api.set("path", path_api)?;

        Ok(api)
    }

    /// Creates the core API table.
    fn core(lua: &Lua) -> mlua::Result<mlua::Table> {
        let api = lua.create_table()?;
        api.set("is_unix", IS_UNIX)?;
        api.set("os", OS)?;

        Ok(api)
    }
}

const IS_UNIX: bool = cfg!(unix);

const OS: &str = os_name();

const fn os_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests;
