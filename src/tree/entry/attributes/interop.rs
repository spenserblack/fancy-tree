//! This module provides utilities for interoperability between Windows and Unix.
use std::fs::Metadata;
use std::path::Path;

/// Checks if the file has the hidden attribute, which is always false on Unix.
#[cfg(not(windows))]
#[inline]
pub fn has_hidden_attribute(_metadata: &Metadata) -> bool {
    false
}

/// Checks if the file has the hidden attribute.
#[cfg(windows)]
pub fn has_hidden_attribute(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    // NOTE See https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;

    let file_attributes = metadata.file_attributes();
    // TODO Test this
    return (file_attributes & FILE_ATTRIBUTE_HIDDEN) != 0;
}

/// Checks if the file has the executable mod set.
#[cfg(not(windows))]
pub fn is_executable<P>(_path: P, metadata: &Metadata) -> bool
where
    P: AsRef<Path>,
{
    use std::os::unix::fs::MetadataExt;

    const OTHERS_HAVE_EXEC: u32 = 0o001;

    // TODO Check if owner or group has execute permission?
    let mode = metadata.mode();
    (mode & OTHERS_HAVE_EXEC) != 0
}

/// Checks if the file's extension is on `%PATHEXT%`.
#[cfg(windows)]
pub fn is_executable<P>(path: P, _metadata: &Metadata) -> bool
where
    P: AsRef<Path>,
{
    use std::collections::HashSet;
    use std::env;
    use std::ffi::{OsStr, OsString};
    use std::sync::LazyLock;

    const KEY: &str = "PATHEXT";
    const SEP: u8 = b';';

    /// Returns a hash set of all the entries in `%PATHEXT%` *normalized to uppercase*.
    #[inline]
    fn get_pathext_hashset() -> HashSet<OsString> {
        let Some(path_exts) = env::var_os(KEY) else {
            return HashSet::new();
        };
        let path_exts = path_exts.as_encoded_bytes();
        let path_exts = path_exts.split(|b| *b == SEP);

        // SAFETY:
        // - Each item only contains content that originated from `OsStr::as_encoded_bytes`.
        let path_exts =
            path_exts.map(|bytes| unsafe { OsStr::from_encoded_bytes_unchecked(bytes) });

        path_exts
            .map(|os_str| os_str.to_ascii_uppercase())
            .collect::<HashSet<_>>()
    }

    /// A set of file executable file extensions. All the entries in the set are
    /// uppercase and have a leading dot (`.`).
    static PATH_EXTS: LazyLock<HashSet<OsString>> = LazyLock::new(get_pathext_hashset);

    let path = path.as_ref();
    let extension = {
        let Some(extension) = path.extension() else {
            return false;
        };
        // NOTE The set is all uppercase, so this needs to be uppercase.
        let extension = extension.to_ascii_uppercase();

        // NOTE `extension()` removes the dot, so we need to add it back.
        //      `%PATHEXT%` entries have leading dots.
        let mut with_dot = OsString::from(".");
        with_dot.push(extension);
        with_dot
    };

    PATH_EXTS.contains(&extension)
}
