//! Module for sorting direction.
use mlua::{FromLua, Lua};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

impl Direction {
    const ASC_PREFIX: &'static str = "asc";
    const DESC_PREFIX: &'static str = "desc";

    /// Converts a string to `Self`.
    fn from_string(s: &mlua::String) -> Option<Self> {
        use Direction::*;

        let s = s.as_bytes();

        [(Self::ASC_PREFIX, Asc), (Self::DESC_PREFIX, Desc)]
            .into_iter()
            .find_map(|(prefix, d)| s.starts_with(prefix.as_bytes()).then_some(d))
    }
}

impl Default for Direction {
    #[inline]
    fn default() -> Self {
        Self::Asc
    }
}

impl FromLua for Direction {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let type_name = value.type_name();

        let conversion_error = || mlua::Error::FromLuaConversionError {
            from: type_name,
            to: String::from("Direction"),
            message: Some(format!(
                r#"Should be either "{}" or "{}""#,
                Self::ASC_PREFIX,
                Self::DESC_PREFIX
            )),
        };

        value
            .as_string()
            .and_then(Self::from_string)
            .ok_or_else(conversion_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(r#""asc""#, Direction::Asc)]
    #[case(r#""ascending""#, Direction::Asc)]
    #[case(r#""desc""#, Direction::Desc)]
    #[case(r#""descending""#, Direction::Desc)]
    fn test_from_lua(#[case] chunk: &str, #[case] expected: Direction) {
        let lua = Lua::new();
        let actual: Direction = lua.load(chunk).eval().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_lua_err() {
        let lua = Lua::new();
        let chunk = r#"1"#;
        assert!(lua.load(chunk).eval::<Direction>().is_err())
    }
}
