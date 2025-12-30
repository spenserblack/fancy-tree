//! Module for sorting paths.
pub use direction::Direction;
pub use directories::Directories;
pub use method::Method;
use mlua::{FromLua, Lua};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::Path;

mod direction;
mod directories;
mod method;

/// Sorting options for paths.
///
/// The sorting priorities are:
///
/// 1. directories
/// 2. method
#[derive(Debug)]
#[non_exhaustive]
pub struct Sorting {
    /// How sorting should be done overall.
    pub method: Method,
    /// The direction to sort.
    pub direction: Direction,
    /// Where to place directories.
    pub directories: Directories,
    /// Whether to ignore case or not.
    ///
    /// Defaults to `false` on Windows, `true` otherwise.
    pub ignore_case: bool,
    /// Ignore the leading dot in dotfiles.
    ///
    /// Assuming case is ignored, here is the difference:
    ///
    /// # Keeping dot
    ///
    /// 1. `.dockerignore`
    /// 2. `.editorconfig`
    /// 3. `Dockerfile`
    ///
    /// # Ignoring dot
    ///
    /// 1. `.dockerignore`
    /// 2. `Dockerfile`
    /// 3. `.editorconfig`
    pub ignore_dot: bool,
}

impl Sorting {
    /// Cleans the dot if necessary.
    fn clean_dot<'a>(&self, os_str: &'a OsStr) -> &'a OsStr {
        if self.ignore_dot {
            let bytes = os_str.as_encoded_bytes();
            let bytes = bytes.strip_prefix(b".").unwrap_or(bytes);
            // SAFETY:
            // - Bytes are all from a valid OsStr and should be safe for conversion.
            unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }
        } else {
            os_str
        }
    }

    /// Cleans up casing for case-insensitive ordering if necessary.
    fn clean_casing<'a>(&self, os_str: &'a OsStr) -> Cow<'a, OsStr> {
        if self.ignore_case {
            Cow::from(os_str.to_ascii_lowercase())
        } else {
            Cow::from(os_str)
        }
    }

    /// Compares two paths.
    pub fn cmp<L, R>(&self, left: L, right: R) -> Ordering
    where
        L: AsRef<Path>,
        R: AsRef<Path>,
    {
        let left = self.clean_dot(left.as_ref().as_os_str());
        let right = self.clean_dot(right.as_ref().as_os_str());
        let left = self.clean_casing(left);
        let right = self.clean_casing(right);

        let ordering = self
            .directories
            .cmp(&left, &right)
            .then_with(|| self.method.cmp(left, right));
        match self.direction {
            Direction::Asc => ordering,
            Direction::Desc => ordering.reverse(),
        }
    }
}

impl Default for Sorting {
    fn default() -> Self {
        let ignore_case = cfg!(windows);
        Self {
            method: Default::default(),
            direction: Default::default(),
            directories: Default::default(),
            ignore_case,
            ignore_dot: false,
        }
    }
}

impl FromLua for Sorting {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        let table = mlua::Table::from_lua(value, lua)?;
        let method = table.get("method")?;
        let direction = table.get("direction")?;
        let directories = table.get("directories")?;
        let ignore_case = table.get("ignore_case")?;
        let ignore_dot = table.get("ignore_dot")?;

        let sorting = Self {
            method,
            direction,
            directories,
            ignore_case,
            ignore_dot,
        };
        Ok(sorting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(false, ".env", ".env")]
    #[case(true, ".env", "env")]
    #[case(true, "foo.txt", "foo.txt")]
    fn test_clean_dot(#[case] ignore_dot: bool, #[case] s: &str, #[case] expected: &str) {
        let sorting = Sorting {
            ignore_dot,
            ..Default::default()
        };

        assert_eq!(OsStr::new(expected), sorting.clean_dot(OsStr::new(s)))
    }

    #[rstest]
    #[case(false, "Dockerfile", "Dockerfile")]
    #[case(true, "Dockerfile", "dockerfile")]
    fn test_clean_casing(#[case] ignore_case: bool, #[case] s: &str, #[case] expected: &str) {
        let sorting = Sorting {
            ignore_case,
            ..Default::default()
        };

        assert_eq!(OsStr::new(expected), sorting.clean_casing(OsStr::new(s)))
    }
}
