//! Module for configuring colors.
use super::ConfigFile;
use crate::color::Color;
use crate::git::status::{self, Status};
use crate::lua::interop;
use crate::tree::{
    Entry,
    entry::{Attributes, attributes::FileAttributes},
};
use mlua::{FromLua, Lua};
use owo_colors::AnsiColors;
use std::path::Path;

/// The configuration for application colors.
#[derive(Debug, Default)]
pub struct Colors {
    /// Function to get the color for an entry's icon.
    for_icon: Option<mlua::Function>,
    git_statuses: GitStatuses,
}

impl Colors {
    /// The default color to use for files.
    const DEFAULT_FILE_COLOR: Option<Color> = None;
    /// The default color to use when a file is an executable.
    const DEFAULT_EXECUTABLE_COLOR: Option<Color> = Some(Color::Ansi(AnsiColors::Green));
    /// The default color to use for directories/folders.
    const DEFAULT_DIRECTORY_COLOR: Option<Color> = Some(Color::Ansi(AnsiColors::Blue));
    /// The default color to use for symlinks.
    const DEFAULT_SYMLINK_COLOR: Option<Color> = Some(Color::Ansi(AnsiColors::Cyan));

    /// Get the color for an entry's icon.
    pub fn for_icon<P>(&self, entry: &Entry<P>) -> Option<Color>
    where
        P: AsRef<Path>,
    {
        let path = entry.path();
        let default: Option<Color> = match entry.attributes() {
            Attributes::Directory(_) => Self::DEFAULT_DIRECTORY_COLOR,
            Attributes::File(attributes) => Self::get_file_color(attributes),
            Attributes::Symlink(_) => Self::DEFAULT_SYMLINK_COLOR,
        };
        let attributes = interop::FileAttributes::from(entry);

        // TODO Report error
        self.for_icon
            .as_ref()
            .map_or(Ok(default), |f| {
                f.call::<Option<Color>>((path, attributes, default))
            })
            .unwrap_or(default)
    }

    /// Get the color for an untracked file's status.
    pub fn for_untracked_git_status(&self, status: Status) -> Option<Color> {
        self.git_statuses.get_untracked_color(status)
    }

    /// Get the color for an tracked file's status.
    pub fn for_tracked_git_status(&self, status: Status) -> Option<Color> {
        self.git_statuses.get_tracked_color(status)
    }

    /// Gets the color for a file.
    fn get_file_color(attributes: &FileAttributes) -> Option<Color> {
        attributes
            .language()
            .map(|language| language.rgb())
            .map(|(r, g, b)| Color::Rgb(r, g, b))
            .or_else(|| {
                attributes
                    .is_executable()
                    .then_some(Self::DEFAULT_EXECUTABLE_COLOR)
                    .flatten()
            })
            .or(Self::DEFAULT_FILE_COLOR)
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
        let for_icon = table.get(FOR_ICON_KEY)?;
        let git_statuses = table
            .get::<Option<GitStatuses>>(GIT_STATUSES_KEY)?
            .unwrap_or_default();

        let colors = Self {
            for_icon,
            git_statuses,
        };
        Ok(colors)
    }
}

/// The configuration for git status colors.
#[derive(Debug, Default)]
struct GitStatuses {
    /// Function to get the color for tracked statuses.
    tracked: Option<mlua::Function>,
    /// Function to get the color for untracked statuses.
    untracked: Option<mlua::Function>,
}

impl GitStatuses {
    /// Gets the default color for a git status.
    const fn get_default_color<S>(status: Status) -> Option<Color>
    where
        S: StatusColor,
    {
        let color = match status {
            Status::Added => S::DEFAULT_ADDED,
            Status::Modified => S::DEFAULT_MODIFIED,
            Status::Removed => S::DEFAULT_REMOVED,
            Status::Renamed => S::DEFAULT_RENAMED,
        };
        Some(Color::Ansi(color))
    }

    /// Gets the color for a tracked git status.
    fn get_tracked_color(&self, status: Status) -> Option<Color> {
        let default = Self::get_default_color::<status::Tracked>(status);
        // TODO Report error
        self.tracked.as_ref().map_or(default, |f| {
            f.call::<Option<Color>>((status, default))
                .unwrap_or(default)
        })
    }

    /// Gets the color for an untracked git status.
    fn get_untracked_color(&self, status: Status) -> Option<Color> {
        let default = Self::get_default_color::<status::Untracked>(status);
        // TODO Report error
        self.untracked.as_ref().map_or(default, |f| {
            f.call::<Option<Color>>((status, default))
                .unwrap_or(default)
        })
    }
}

impl FromLua for GitStatuses {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        const TRACKED_KEY: &str = "tracked";
        const UNTRACKED_KEY: &str = "untracked";

        let table = mlua::Table::from_lua(value, lua)?;
        let tracked = table.get(TRACKED_KEY)?;
        let untracked = table.get(UNTRACKED_KEY)?;

        let git_statuses = Self { tracked, untracked };
        Ok(git_statuses)
    }
}

/// Private trait to generalize getting the color for a status.
trait StatusColor {
    /// Default color for added status.
    const DEFAULT_ADDED: AnsiColors;
    /// Default color for modified status.
    const DEFAULT_MODIFIED: AnsiColors;
    /// Default color for removed status.
    const DEFAULT_REMOVED: AnsiColors;
    /// Default color for renamed status.
    const DEFAULT_RENAMED: AnsiColors;
}

impl StatusColor for status::Tracked {
    const DEFAULT_ADDED: AnsiColors = AnsiColors::Green;
    const DEFAULT_MODIFIED: AnsiColors = AnsiColors::Yellow;
    const DEFAULT_REMOVED: AnsiColors = AnsiColors::Red;
    const DEFAULT_RENAMED: AnsiColors = AnsiColors::Cyan;
}

impl StatusColor for status::Untracked {
    const DEFAULT_ADDED: AnsiColors = AnsiColors::BrightGreen;
    const DEFAULT_MODIFIED: AnsiColors = AnsiColors::BrightYellow;
    const DEFAULT_REMOVED: AnsiColors = AnsiColors::BrightRed;
    const DEFAULT_RENAMED: AnsiColors = AnsiColors::BrightCyan;
}
