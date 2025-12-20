//! Module for configuring colors.
use super::ConfigFile;
use crate::color::Color;
use crate::git::status::Status;
use crate::lua::interop;
use crate::tree::Entry;
use mlua::{FromLua, Lua};
use std::path::Path;

/// The configuration for application colors.
#[derive(Debug)]
pub struct Colors {
    /// Function to get the color for an entry's icon.
    for_icon: mlua::Function,
    git_statuses: GitStatuses,
}

impl Colors {
    /// Get the color for an entry's icon.
    pub fn for_icon<P>(
        &self,
        entry: &Entry<P>,
        default_choice: Option<Color>,
    ) -> mlua::Result<Option<Color>>
    where
        P: AsRef<Path>,
    {
        let path = entry.path();
        let attributes = interop::FileAttributes::from(entry);

        self.for_icon.call((path, attributes, default_choice))
    }

    /// Get the color for an untracked file's status.
    pub fn for_untracked_git_status(&self, status: Status, default_choice: Option<Color>) -> mlua::Result<Option<Color>> {
        self.git_statuses.untracked.call((status, default_choice))
    }

    /// Get the color for an tracked file's status.
    pub fn for_tracked_git_status(&self, status: Status, default_choice: Option<Color>) -> mlua::Result<Option<Color>> {
        self.git_statuses.tracked.call((status, default_choice))
    }
}

impl ConfigFile for Colors {
    const FILENAME: &'static str = "colors.lua";
    const DEFAULT_MODULE: &'static str = include_str!("./colors.lua");
}

impl FromLua for Colors {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        const FOR_ICON_KEY: &str = "icons";
        const GIT_STATUSES_KEY: &str = "git_statuses";

        let table = mlua::Table::from_lua(value, lua)?;
        let for_icon = table.get::<mlua::Function>(FOR_ICON_KEY)?;
        let git_statuses = table.get::<GitStatuses>(GIT_STATUSES_KEY)?;

        let colors = Self {
            for_icon,
            git_statuses,
        };
        Ok(colors)
    }
}

/// The configuration for git status colors.
#[derive(Debug)]
struct GitStatuses {
    /// Function to get the color for tracked statuses.
    tracked: mlua::Function,
    /// Function to get the color for untracked statuses.
    untracked: mlua::Function,
}

impl FromLua for GitStatuses {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        const TRACKED_KEY: &str = "tracked";
        const UNTRACKED_KEY: &str = "untracked";

        let table = mlua::Table::from_lua(value, lua)?;
        let tracked = table.get::<mlua::Function>(TRACKED_KEY)?;
        let untracked = table.get::<mlua::Function>(UNTRACKED_KEY)?;

        let git_statuses = Self { tracked, untracked };
        Ok(git_statuses)
    }
}
