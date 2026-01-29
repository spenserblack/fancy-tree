//! Module for the icon config.
use super::ConfigFile;
use crate::icons;
use crate::lua::interop;
use crate::tree::{
    Entry,
    entry::{Attributes, attributes::FileAttributes},
};
use mlua::{FromLua, Lua};
use std::path::Path;

/// The configuration for icons.
#[derive(Debug, Default)]
pub struct Icons {
    /// Function to get the icon for an entry.
    get_icon: Option<mlua::Function>,
}

impl Icons {
    /// The default icon to display for files.
    const DEFAULT_FILE_ICON: &'static str = "\u{f0214}"; // 󰈔
    /// The default icon to display when a file is an executable.
    const DEFAULT_EXECUTABLE_ICON: &'static str = "\u{f070e}"; // 󰜎
    /// The default icon to display for directories/folders.
    const DEFAULT_DIRECTORY_ICON: &'static str = "\u{f024b}"; // 󰉋
    /// The default icon to display for symlinks.
    const DEFAULT_SYMLINK_ICON: &'static str = "\u{cf481}"; // 

    /// The icon (padding) to use if there is no icon.
    const EMPTY_ICON: &'static str = " ";

    /// Get the icon for the entry. If the configuration returns `nil`, a string with
    /// invisible characters will be returned.
    ///
    /// On a Lua error, this falls back to the default icon choice.
    pub fn get_icon<P>(&self, entry: &Entry<P>) -> String
    where
        P: AsRef<Path>,
    {
        // TODO Use Cow
        let default_icon =
            icons::for_path(entry.path()).unwrap_or_else(|| Self::default_icon(entry));
        self.get_icon
            .as_ref()
            .and_then(|f| {
                let path = entry.path();
                let attributes = interop::FileAttributes::from(entry);
                // TODO Report the error when this function fails
                f.call::<Option<String>>((path, attributes, default_icon))
                    .ok()
            })
            .unwrap_or_else(|| Some(String::from(default_icon)))
            .unwrap_or_else(|| String::from(Self::EMPTY_ICON))
    }

    /// Gets the default icon choice for an entry.
    fn default_icon<P>(entry: &Entry<P>) -> &str
    where
        P: AsRef<Path>,
    {
        match entry.attributes() {
            Attributes::Directory(_) => Self::DEFAULT_DIRECTORY_ICON,
            Attributes::File(attributes) => Self::get_file_icon(attributes),
            Attributes::Symlink(_) => Self::DEFAULT_SYMLINK_ICON,
        }
    }

    /// Gets the default icon for a file entry.
    fn get_file_icon(attributes: &FileAttributes) -> &'static str {
        if attributes.is_executable() {
            return Self::DEFAULT_EXECUTABLE_ICON;
        }
        attributes
            .language()
            .and_then(|language| language.nerd_font_glyph())
            .unwrap_or(Self::DEFAULT_FILE_ICON)
    }
}

impl ConfigFile for Icons {
    const FILENAME: &'static str = "icons.lua";
    const DEFAULT_MODULE: &'static str = include_str!("./icons.lua");
}

impl FromLua for Icons {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        Option::<mlua::Function>::from_lua(value, lua).map(|get_icon| Self { get_icon })
    }
}
