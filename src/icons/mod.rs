//! Provides icons for filepaths.
use crate::ext::PathExt as _;
use std::path::Path;
use std::sync::LazyLock;

/// Gets an icon for a path.
pub fn for_path<P>(path: P) -> Option<&'static str>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    path.file_name()
        .and_then(|s| s.to_str())
        .and_then(for_filename)
        .or_else(|| {
            path.double_extension()
                .and_then(|(prefix, suffix)| {
                    prefix
                        .to_str()
                        .and_then(|prefix| suffix.to_str().map(|suffix| (prefix, suffix)))
                })
                .and_then(for_double_extension)
        })
        .or_else(|| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .and_then(for_extension)
        })
        .or_else(|| for_filename_glob(path))
}

/// Gets an icon for a filename.
fn for_filename(filename: &str) -> Option<&'static str> {
    // NOTE These should be in alphabetical order and ignoring any leading `.` for
    //      easier code review.
    let icon = match filename {
        "CONTRIBUTING.md" => shared::DOC,
        ".editorconfig" => "\u{e652}", // 
        ".git" => "\u{e702}",          // 
        ".github" => "\u{e709}",       // 
        "LICENCE" | "LICENSE" | "licence" | "license" => shared::LICENSE,
        "package-lock.json" | "pnpm-lock.yaml" => shared::LOCK,
        "README" | "README.md" => shared::DOC,
        ".vscode" => "\u{e8da}", // 
        _ => return None,
    };
    Some(icon)
}

/// Gets an icon for a file extension.
fn for_extension(extension: &str) -> Option<&'static str> {
    // NOTE These should be in alphabetical order for easier code review.
    let icon = match extension {
        "cfg" => "\u{e615}", // 
        "gif" | "jpeg" | "jpg" | "png" => shared::IMAGE,
        "lock" => shared::LOCK,
        _ => return None,
    };

    Some(icon)
}

/// Gets an icon for the double extension.
fn for_double_extension(double_extension: (&str, &str)) -> Option<&'static str> {
    let color = match double_extension {
        ("tar", "gz") => shared::ARCHIVE,
        _ => return None,
    };

    Some(color)
}

/// Gets an icon based on a matching glob for a path.
fn for_filename_glob(path: &Path) -> Option<&'static str> {
    use glob::{MatchOptions, Pattern};

    /// Maps a raw glob pattern to an icon with `(glob, icon)` tuples.
    const RAW_MAPPINGS: &[(&str, &str)] = &[("LICEN[CS]E-*", shared::LICENSE)];

    const OPTIONS: MatchOptions = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    /// The compiled glob-to-icon mappings.
    static COMPILED_MAPPINGS: LazyLock<Vec<(Pattern, &'static str)>> = LazyLock::new(|| {
        RAW_MAPPINGS
            .iter()
            .map(|(raw, icon)| (Pattern::new(raw).expect("Pattern should be valid"), *icon))
            .collect()
    });

    // NOTE This may receive a path with `./`, so we'll clean to just the prefix.
    path.file_name().and_then(|s| s.to_str()).and_then(|path| {
        COMPILED_MAPPINGS
            .iter()
            .find_map(|(glob, icon)| glob.matches_with(path, OPTIONS).then_some(*icon))
    })
}

/// Icons that represent one file type, but have multiple filenames and/or extensions
/// for that file type.
mod shared {
    /// Icon for archive files, like `.zip` or `.tar.gz`.
    pub const ARCHIVE: &str = "\u{ea98}"; // 
    /// Icon for documentation files, like READMEs.
    pub const DOC: &str = "\u{eaa4}"; // 
    /// Icon for license files.
    pub const LICENSE: &str = "\u{e60a}"; // 
    /// Icon for lock files.
    pub const LOCK: &str = "\u{e672}"; // 
    /// Icon for image files.
    pub const IMAGE: &str = "\u{f1c5}"; // 
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("example.tar.gz", Some(shared::ARCHIVE))]
    #[case("example.gif", Some(shared::IMAGE))]
    #[case("example.jpeg", Some(shared::IMAGE))]
    #[case("example.jpg", Some(shared::IMAGE))]
    #[case("example.png", Some(shared::IMAGE))]
    fn test_for_path<P>(#[case] path: P, #[case] expected: Option<&str>)
    where
        P: AsRef<Path>,
    {
        assert_eq!(expected, for_path(path));
    }
}
