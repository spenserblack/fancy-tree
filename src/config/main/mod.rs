//! Module for the main config.
use super::ConfigFile;
use crate::color::ColorChoice;
use crate::lua::interop;
use crate::sorting;
use crate::tree::Entry;
use mlua::{
    Either::{self, Left, Right},
    FromLua, Lua,
};
use std::cmp::Ordering;
use std::path::Path;

/// Either a sorting configuration, or a function that takes two values and returns
/// a negative number for less-than, 0 for equal, or a positive number for greater-than.
type Sorting = Either<sorting::Sorting, mlua::Function>;

/// The main configuration type.
#[derive(Debug, Default)]
pub struct Main {
    /// Determines when/how the application should show colors.
    color: ColorChoice,
    /// Function to determine if a file should be skipped.
    skip: Option<mlua::Function>,
    /// Determines how to sort files in a directory.
    sorting: Sorting,
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

    /// Compares two paths for sorting.
    pub fn cmp<L, R>(&self, left: L, right: R) -> mlua::Result<Ordering>
    where
        L: AsRef<Path>,
        R: AsRef<Path>,
    {
        match self.sorting.as_ref() {
            Left(sorting) => Ok(sorting.cmp(left, right)),
            Right(f) => f
                .call((left.as_ref(), right.as_ref()))
                .map(Self::isize_to_ordering),
        }
    }

    /// Creates the default sorting configuration.
    fn default_sorting() -> Sorting {
        Left(Default::default())
    }

    /// Converts a number returned by a lua function for comparing paths into [`Ordering`].
    fn isize_to_ordering(n: isize) -> Ordering {
        match n {
            ..=-1 => Ordering::Less,
            0 => Ordering::Equal,
            1.. => Ordering::Greater,
        }
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
        let sorting = table
            .get::<Option<Sorting>>("sorting")?
            .unwrap_or_else(Self::default_sorting);
        let main = Main {
            color,
            skip,
            sorting,
        };
        Ok(main)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(-1, Ordering::Less)]
    #[case(0, Ordering::Equal)]
    #[case(1, Ordering::Greater)]
    fn test_isize_to_ordering(#[case] n: isize, #[case] expected: Ordering) {
        assert_eq!(expected, Main::isize_to_ordering(n));
    }
}
