//! This module provides utilities for colorization.
pub use choice::ColorChoice;
use either::{Either, Left, Right};
use mlua::{FromLua, IntoLua, Lua};
use owo_colors::{
    AnsiColors::{
        self, Black, Blue, BrightBlack, BrightBlue, BrightCyan, BrightGreen, BrightMagenta,
        BrightRed, BrightWhite, BrightYellow, Cyan, Green, Magenta, Red, White, Yellow,
    },
    DynColors,
};

mod choice;

/// Either ANSI colors or full RGB.
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Ansi(AnsiColors),
    Rgb(u8, u8, u8),
}

impl Color {
    /// Maps ansi color names to their values.
    const ANSI_NAME_MAP: [(&'static str, AnsiColors); 16] = [
        ("black", Black),
        ("red", Red),
        ("green", Green),
        ("yellow", Yellow),
        ("blue", Blue),
        ("magenta", Magenta),
        ("cyan", Cyan),
        ("white", White),
        ("bright-black", BrightBlack),
        ("bright-red", BrightRed),
        ("bright-green", BrightGreen),
        ("bright-yellow", BrightYellow),
        ("bright-blue", BrightBlue),
        ("bright-magenta", BrightMagenta),
        ("bright-cyan", BrightCyan),
        ("bright-white", BrightWhite),
    ];

    /// Tries to create an Ansi color from Lua.
    fn ansi_from_lua_string(type_name: &'static str, s: &str) -> mlua::Result<Self> {
        Self::ANSI_NAME_MAP
            .into_iter()
            .find_map(|(key, value)| (key == s).then_some(value))
            .map(Self::Ansi)
            .ok_or(mlua::Error::FromLuaConversionError {
                from: type_name,
                to: String::from("Color"),
                message: Some(String::from("Expected one of the ansi color names")),
            })
    }

    /// Tries to create an Rgb color from Lua.
    fn rgb_from_lua_table(t: mlua::Table) -> mlua::Result<Self> {
        // let [r, g, b] = ["r", "g", "b"].map(|key| t.get::<u8>(key));
        t.get::<u8>("r")
            .and_then(|r| t.get::<u8>("g").map(|g| (r, g)))
            .and_then(|(r, g)| t.get::<u8>("b").map(|b| (r, g, b)))
            .map(|(r, g, b)| Self::Rgb(r, g, b))
    }

    /// Gets the key name for the ANSI color.
    fn ansi_name(ansi_colors: AnsiColors) -> &'static str {
        debug_assert!(
            Self::ANSI_NAME_MAP
                .into_iter()
                .any(|(_, color)| color == ansi_colors)
        );

        Self::ANSI_NAME_MAP
            .into_iter()
            .find_map(|(key, value)| (value == ansi_colors).then_some(key))
            .expect("The mapping should exist")
    }

    /// Converts RGB into a table.
    #[inline]
    fn rgb_to_table(lua: &Lua, r: u8, g: u8, b: u8) -> mlua::Result<mlua::Table> {
        lua.create_table_from([("r", r), ("g", g), ("b", b)])
    }
}

impl FromLua for Color {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        type AnsiOrRgb = Either<String, mlua::Table>;
        let type_name = value.type_name();
        let ansi_or_rgb = AnsiOrRgb::from_lua(value, lua)?;

        match ansi_or_rgb {
            Left(ansi) => Self::ansi_from_lua_string(type_name, &ansi),
            Right(table) => Self::rgb_from_lua_table(table),
        }
    }
}

impl IntoLua for Color {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        match self {
            Color::Ansi(ansi_colors) => Color::ansi_name(ansi_colors).into_lua(lua),
            Color::Rgb(r, g, b) => Color::rgb_to_table(lua, r, g, b)?.into_lua(lua),
        }
    }
}

impl From<Color> for DynColors {
    fn from(value: Color) -> Self {
        match value {
            Color::Ansi(color) => Self::Ansi(color),
            Color::Rgb(r, g, b) => Self::Rgb(r, g, b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("black", AnsiColors::Black)]
    #[case("magenta", AnsiColors::Magenta)]
    #[case("bright-red", AnsiColors::BrightRed)]
    #[case("bright-yellow", AnsiColors::BrightYellow)]
    fn test_from_lua_string_ok(#[case] raw: &str, #[case] expected_ansi: AnsiColors) {
        let lua = Lua::new();
        let value = lua.create_string(raw).expect("A string to be created");
        let value = mlua::Value::String(value);
        let color = Color::from_lua(value, &lua).expect("Color should be converted");
        let Color::Ansi(ansi) = color else {
            panic!("Expected Color::Ansi")
        };
        assert_eq!(expected_ansi, ansi);
    }

    #[test]
    fn test_from_lua_string_err() {
        let lua = Lua::new();
        let value = lua
            .create_string("??unused??")
            .expect("A string to be created");
        let value = mlua::Value::String(value);
        assert!(Color::from_lua(value, &lua).is_err());
    }

    #[test]
    fn test_from_lua_tuple_ok() {
        let lua = Lua::new();
        let value = lua
            .create_table_from([("r", 255u8), ("g", 0), ("b", 128)])
            .expect("A table should be created");
        let value = mlua::Value::Table(value);
        let color = Color::from_lua(value, &lua).expect("Color should be converted");
        let Color::Rgb(r, g, b) = color else {
            panic!("Expected Color::Rgb")
        };
        assert_eq!(255, r);
        assert_eq!(0, g);
        assert_eq!(128, b);
    }

    #[test]
    fn test_from_lua_tuple_err() {
        let lua = Lua::new();
        let value = lua
            .create_table_from([("r", 255u8), ("b", 0)])
            .expect("A table should be created");
        let value = mlua::Value::Table(value);
        assert!(Color::from_lua(value, &lua).is_err());
    }
}
