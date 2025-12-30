//! Module for how to include directories in sorting.
use mlua::{FromLua, Lua};
use std::cmp::Ordering;
use std::path::Path;

/// How directories should be included in sorting.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Directories {
    /// Directories and files should be mixed together.
    Mixed,
    /// Directories should come first.
    First,
    /// Directories should be last.
    Last,
}

impl Directories {
    const MIXED_NAME: &'static str = "mixed";
    const FIRST_NAME: &'static str = "first";
    const LAST_NAME: &'static str = "last";

    /// Converts a string to `Self`.
    fn from_string(s: &str) -> Option<Self> {
        use Directories::*;

        [
            (Self::MIXED_NAME, Mixed),
            (Self::FIRST_NAME, First),
            (Self::LAST_NAME, Last),
        ]
        .into_iter()
        .find_map(|(name, d)| (s == name).then_some(d))
    }

    /// Compares two paths and provides the proper ordering if they are directories or not.
    pub fn cmp<L, R>(&self, left: L, right: R) -> Ordering
    where
        L: AsRef<Path>,
        R: AsRef<Path>,
    {
        self.cmp_impl(left.as_ref(), right.as_ref())
    }

    /// Implementation for comparison.
    fn cmp_impl<L, R>(&self, left: L, right: R) -> Ordering
    where
        L: IsDirectory,
        R: IsDirectory,
    {
        if let Self::Mixed = self {
            // NOTE Whether or not they are directories don't matter at all. Small
            //      optimization to avoid calling path methods.
            return Ordering::Equal;
        }

        match (self, right.is_directory(), left.is_directory()) {
            (Self::Mixed, _, _) => unreachable!("Already checked for mixed ordering"),
            (_, true, true) | (_, false, false) => Ordering::Equal,
            (Self::First, true, false) | (Self::Last, false, true) => Ordering::Less,
            (Self::First, false, true) | (Self::Last, true, false) => Ordering::Greater,
        }
    }
}

impl Default for Directories {
    #[inline]
    fn default() -> Self {
        Self::Mixed
    }
}

impl FromLua for Directories {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        let type_name = value.type_name();

        let conversion_error = || {
            let choices = [Self::MIXED_NAME, Self::FIRST_NAME, Self::LAST_NAME].join(", ");

            mlua::Error::FromLuaConversionError {
                from: type_name,
                to: String::from("Directories"),
                message: Some(choices),
            }
        };

        let s = String::from_lua(value, lua)?;
        Self::from_string(&s).ok_or_else(conversion_error)
    }
}

/// Trait for types to check if they are directories.
trait IsDirectory {
    /// Returns true if the thing is a directory.
    fn is_directory(&self) -> bool;
}

impl IsDirectory for &Path {
    #[inline]
    fn is_directory(&self) -> bool {
        self.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(r#""mixed""#, Directories::Mixed)]
    #[case(r#""first""#, Directories::First)]
    #[case(r#""last""#, Directories::Last)]
    fn test_from_lua(#[case] chunk: &str, #[case] expected: Directories) {
        let lua = Lua::new();
        let actual: Directories = lua.load(chunk).eval().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_lua_err() {
        let lua = Lua::new();
        let chunk = r#"1"#;
        assert!(lua.load(chunk).eval::<Directories>().is_err())
    }

    #[rstest]
    #[case(Directories::Mixed, true, false, Ordering::Equal)]
    #[case(Directories::Mixed, false, true, Ordering::Equal)]
    #[case(Directories::First, true, true, Ordering::Equal)]
    #[case(Directories::First, true, false, Ordering::Greater)]
    #[case(Directories::First, false, true, Ordering::Less)]
    #[case(Directories::Last, true, true, Ordering::Equal)]
    #[case(Directories::Last, true, false, Ordering::Less)]
    #[case(Directories::Last, false, true, Ordering::Greater)]
    fn test_cmp(
        #[case] directories: Directories,
        #[case] left_is_dir: bool,
        #[case] right_is_dir: bool,
        #[case] expected: Ordering,
    ) {
        struct IsDir(bool);
        impl IsDirectory for IsDir {
            fn is_directory(&self) -> bool {
                self.0
            }
        }

        assert_eq!(
            expected,
            directories.cmp_impl(IsDir(left_is_dir), IsDir(right_is_dir))
        )
    }
}
