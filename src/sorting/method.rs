//! Module for the sorting method.

use mlua::{FromLua, Lua};
use std::cmp::Ordering;
use std::ffi::OsStr;

/// How items should be sorted.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Method {
    /// Compare and sort by value. Like alphabetical sorting, but special characters
    /// are also considered for sorting.
    Naive,
    /// Number strings are parsed and compared within filenames. This means that
    /// `notes-10.txt` comes *after* `notes-2.txt`, not before.
    Natural,
}

impl Method {
    const NAIVE_NAME: &'static str = "naive";
    const NATURAL_NAME: &'static str = "natural";

    /// Compares two OS strings.
    pub fn cmp<L, R>(&self, left: L, right: R) -> Ordering
    where
        L: AsRef<OsStr>,
        R: AsRef<OsStr>,
    {
        let left = left.as_ref();
        let right = right.as_ref();

        match self {
            Self::Naive => left.cmp(right),
            Self::Natural => Self::cmp_natural(left, right),
        }
    }

    /// Naturally sort two OS strings.
    fn cmp_natural(left: &OsStr, right: &OsStr) -> Ordering {
        let mut left = left.as_encoded_bytes().iter().copied();
        let mut right = right.as_encoded_bytes().iter().copied();
        loop {
            let (left_char, right_char) = match (left.next(), right.next()) {
                (None, None) => break Ordering::Equal,
                (Some(_), None) => break Ordering::Greater,
                (None, Some(_)) => break Ordering::Less,
                (Some(left), Some(right)) => (left, right),
            };
            if !(left_char.is_ascii_digit() && right_char.is_ascii_digit()) {
                // NOTE Cannot order numerically, falling back to basic ordering.
                if let Some(ordering) = Self::cmp_natural_bytes(left_char, right_char) {
                    break ordering;
                }
                continue;
            }
            // NOTE Both are ASCII digits, we should consume and compare.
            let left = Self::consume_digits(left_char, &mut left);
            let right = Self::consume_digits(right_char, &mut right);
            let comparison = left.cmp(&right);
            if comparison.is_ne() {
                break comparison;
            }
        }
    }

    /// Compares two bytes for natural sorting, returning `Some(Ordering)` if the order
    /// of the providing strings can be determined. Returns `None` if the next set of
    /// bytes needs to be checked. Never returns `Ordering::Equal`. The left and right
    /// values must be from the same index.
    fn cmp_natural_bytes(left: u8, right: u8) -> Option<Ordering> {
        let ordering = left.cmp(&right);
        if ordering.is_eq() {
            None
        } else {
            Some(ordering)
        }
    }

    /// Consumes part of a byte iterator to get a numerical string. The first char is the
    /// "trigger" to call this, and should be prepended.
    fn consume_digits<I>(first_digit: u8, bytes: I) -> usize
    where
        I: Iterator<Item = u8>,
    {
        let remaining_digits = bytes.take_while(|b| b.is_ascii_digit());
        let digits = [first_digit]
            .into_iter()
            .chain(remaining_digits)
            .collect::<Vec<u8>>();
        // TODO If we're 100% confident, we can use the unsafe `from_utf8_unchecked` method.
        let digits = String::from_utf8(digits).expect("The digits should all be valid UTF-8");
        digits.parse().expect("The string should be a valid number")
    }

    /// Converts a string to `Self`.
    fn from_string(s: &str) -> Option<Self> {
        use Method::*;

        [(Self::NAIVE_NAME, Naive), (Self::NATURAL_NAME, Natural)]
            .into_iter()
            .find_map(|(name, m)| (s == name).then_some(m))
    }
}

impl Default for Method {
    #[inline]
    fn default() -> Self {
        Self::Naive
    }
}

impl FromLua for Method {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        let type_name = value.type_name();

        let conversion_error = || mlua::Error::FromLuaConversionError {
            from: type_name,
            to: String::from("Directories"),
            message: Some(format!(
                r#"Should be either "{}" or "{}""#,
                Self::NAIVE_NAME,
                Self::NATURAL_NAME
            )),
        };

        let s = String::from_lua(value, lua)?;
        Self::from_string(&s).ok_or_else(conversion_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::naive(Method::Naive, "a", "b", Ordering::Less)]
    #[case::naive(Method::Naive, "2", "10", Ordering::Greater)]
    #[case::natural(Method::Natural, "a", "b", Ordering::Less)]
    #[case::natural(Method::Natural, "2", "10", Ordering::Less)]
    #[case::natural(Method::Natural, "2.txt", "10.txt", Ordering::Less)]
    #[case::natural(Method::Natural, "12.txt", "10.txt", Ordering::Greater)]
    #[case::natural(Method::Natural, "1-2.txt", "10.txt", Ordering::Less)]
    #[case::natural(Method::Natural, "100-a.txt", "100-b.txt", Ordering::Less)]
    fn test_cmp(
        #[case] method: Method,
        #[case] left: &str,
        #[case] right: &str,
        #[case] expected: Ordering,
    ) {
        assert_eq!(expected, method.cmp(left, right))
    }

    #[rstest]
    #[case(r#""naive""#, Method::Naive)]
    #[case(r#""natural""#, Method::Natural)]
    fn test_from_lua(#[case] chunk: &str, #[case] expected: Method) {
        let lua = Lua::new();
        let actual: Method = lua.load(chunk).eval().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_lua_err() {
        let lua = Lua::new();
        let chunk = r#"1"#;
        assert!(lua.load(chunk).eval::<Method>().is_err())
    }
}
