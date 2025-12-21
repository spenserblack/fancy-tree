//! Module for miscellaneous utilities for Git.
use git2::StatusEntry;
use std::path::PathBuf;

/// Trait to get a path from a `git2::StatusEntry`.
pub trait StatusEntryExt: sealed::Sealed {
    /// Gets a `PathBuf` for a `StatusEntry`.
    fn path_buf(&self) -> Option<PathBuf>;
}

#[cfg(unix)]
impl<'a> StatusEntryExt for StatusEntry<'a> {
    /// Creates a `PathBuf` from this status entry's path. Always returns `Some`.
    fn path_buf(&self) -> Option<PathBuf> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path = self.path_bytes();
        let path = OsStr::from_bytes(path);
        let path = PathBuf::from(path);
        Some(path)
    }
}

#[cfg(not(unix))]
impl<'a> StatusEntryExt for StatusEntry<'a> {
    fn path_buf(&self) -> Option<PathBuf> {
        self.path().map(PathBuf::from)
    }
}

/// Private module for `Sealed`.
mod sealed {
    use git2::StatusEntry;

    /// Seals `StatusEntryExt` for `StatusEntry`.
    pub trait Sealed {}

    impl<'a> Sealed for StatusEntry<'a> {}
}
