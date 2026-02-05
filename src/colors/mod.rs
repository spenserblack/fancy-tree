//! Provides colors for filepaths.
use crate::color::Color;
use owo_colors::AnsiColors::{Black, Blue, Cyan, Green, Red, Yellow};
use std::path::Path;
use std::sync::LazyLock;

/// Gets a color for a path.
pub fn for_path<P>(path: P) -> Option<Color>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    path.file_name()
        .and_then(|s| s.to_str())
        .and_then(for_filename)
        .or_else(|| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .and_then(for_extension)
        })
        .or_else(|| for_filename_glob(path))
}

/// Gets a color for a filename.
fn for_filename(filename: &str) -> Option<Color> {
    // NOTE These should be in alphabetical order and ignoring any leading `.` for
    //      easier code review.
    let color = match filename {
        ".git" => Red.into(),
        ".github" => Black.into(),
        ".vscode" => Blue.into(),
        _ => return None,
    };
    Some(color)
}

/// Gets a color for a file extension.
fn for_extension(extension: &str) -> Option<Color> {
    // NOTE These should be in alphabetical order for easier code review.
    let color = match extension {
        "gif" => Green.into(),
        "jpeg" | "jpg" => Yellow.into(),
        "png" => Cyan.into(),
        _ => return None,
    };

    Some(color)
}

/// Gets a color based on a matching glob for a path.
fn for_filename_glob(path: &Path) -> Option<Color> {
    use glob::{MatchOptions, Pattern};

    /// Maps a raw glob pattern to a color with `(glob, color)` tuples.
    const RAW_MAPPINGS: &[(&str, Color)] = &[("LICEN[CS]E-*", Color::Ansi(Yellow))];

    const OPTIONS: MatchOptions = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    /// The compiled glob-to-color mappings.
    static COMPILED_MAPPINGS: LazyLock<Vec<(Pattern, Color)>> = LazyLock::new(|| {
        RAW_MAPPINGS
            .iter()
            .map(|(raw, color)| (Pattern::new(raw).expect("Pattern should be valid"), *color))
            .collect()
    });

    // NOTE This may receive a path with `./`, so we'll clean to just the prefix.
    path.file_name().and_then(|s| s.to_str()).and_then(|path| {
        COMPILED_MAPPINGS
            .iter()
            .find_map(|(glob, color)| glob.matches_with(path, OPTIONS).then_some(*color))
    })
}
