//! This module provides utilities for configuration files.
use crate::Result;
pub use colors::Colors;
use directories::ProjectDirs;
pub use icons::Icons;
pub use main::Main;
use mlua::{FromLuaMulti, Lua};
use std::fs;
use std::path::{Path, PathBuf};

mod colors;
mod icons;
mod main;

/// The project configuration directory.
pub struct ConfigDir {
    /// Utility for finding the project directories.
    project_dirs: ProjectDirs,
}

impl ConfigDir {
    /// The project's qualifier (empty).
    const QUALIFIER: &str = "";
    /// The project's organization (none).
    const ORGANIZATION: &str = "";
    /// The project name.
    const APPLICATION: &str = env!("CARGO_PKG_NAME");

    /// The directory containing the config files.
    pub fn new() -> Result<Self, &'static str> {
        ProjectDirs::from(Self::QUALIFIER, Self::ORGANIZATION, Self::APPLICATION)
            .map(|project_dirs| Self { project_dirs })
            .ok_or("Missing home directory")
    }

    /// Creates the configuration directory if it doesn't exist.
    pub fn create_dir(&self) -> Result {
        let dir = self.path();
        fs::create_dir_all(dir)?;
        Ok(())
    }

    /// Loads the main configuration file.
    #[inline]
    pub fn load_main(&self, lua: &Lua) -> mlua::Result<Option<Main>> {
        self.load_file(lua)
    }

    /// Loads the icon configuration file.
    #[inline]
    pub fn load_icons(&self, lua: &Lua) -> mlua::Result<Option<Icons>> {
        self.load_file(lua)
    }

    /// Loads the colors configuration file.
    #[inline]
    pub fn load_colors(&self, lua: &Lua) -> mlua::Result<Option<Colors>> {
        self.load_file(lua)
    }

    /// Loads a `.lua` file from the configuration directory.
    fn load_file<T>(&self, lua: &Lua) -> mlua::Result<Option<T>>
    where
        T: ConfigFile + FromLuaMulti,
    {
        let path = self.path().join(T::FILENAME);
        path.exists()
            .then(|| {
                let chunk = lua.load(path);
                chunk.call::<T>(())
            })
            .transpose()
    }

    /// Gets the config directory for the project.
    #[inline]
    pub fn path(&self) -> &Path {
        self.project_dirs.config_dir()
    }

    /// Gets the path of a file in the configuration directory from its filename.
    fn file_name<T>(&self) -> PathBuf
    where
        T: ConfigFile,
    {
        self.path().join(T::FILENAME)
    }

    /// Gets the path to the main configuration file.
    #[inline]
    pub fn main_path(&self) -> PathBuf {
        self.file_name::<Main>()
    }

    /// Gets the path to the icons configuration file.
    #[inline]
    pub fn icons_path(&self) -> PathBuf {
        self.file_name::<Icons>()
    }

    /// Gets the path to the colors configuration file.
    #[inline]
    pub fn colors_path(&self) -> PathBuf {
        self.file_name::<Colors>()
    }
}

/// Common behavior for configuration files.
pub trait ConfigFile {
    /// The filename in the configuration directory.
    const FILENAME: &'static str;
    /// The default lua module.
    const DEFAULT_MODULE: &'static str;
}
