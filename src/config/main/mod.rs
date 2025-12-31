//! Module for the main config.
use super::ConfigFile;
use crate::color::ColorChoice;
use crate::lua::interop;
use crate::tree::Entry;
use mlua::{FromLua, Lua};
use std::path::Path;

/// The main configuration type.
#[derive(Debug, Default)]
pub struct Main {
    /// Determines when/how the application should show colors.
    color: ColorChoice,
    /// Function to determine if a file should be skipped.
    skip: Option<mlua::Function>,
}

impl Main {
    /// Gets the configured color choice.
    #[inline]
    pub fn color_choice(&self) -> ColorChoice {
        self.color
    }
    /// Should a file be skipped according to the configuration?
    ///
    /// `git_helper` is used to provide interoperability with git, which this config
    /// type isn't aware of.
    pub fn should_skip<P, F>(&self, entry: &Entry<P>, git_helper: F) -> bool
    where
        P: AsRef<Path>,
        F: FnOnce() -> bool,
    {
        let default = entry.is_hidden() || git_helper();
        let path = entry.path();
        let attributes = interop::FileAttributes::from(entry);

        // TODO Report error
        self.skip
            .as_ref()
            .map_or(Ok(default), |f| f.call::<bool>((path, attributes, default)))
            .unwrap_or(default)
    }
}

impl ConfigFile for Main {
    const FILENAME: &'static str = "config.lua";
    const DEFAULT_MODULE: &'static str = include_str!("./config.lua");
}

impl FromLua for Main {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let type_name = value.type_name();

        let conversion_error = || mlua::Error::FromLuaConversionError {
            from: type_name,
            to: String::from("config::Main"),
            message: None,
        };

        let table = value.as_table().ok_or_else(conversion_error)?;
        let color = table
            .get::<Option<ColorChoice>>("color")?
            .unwrap_or_default();
        let skip: Option<mlua::Function> = table.get("skip")?;
        let main = Main { color, skip };
        Ok(main)
    }
}
