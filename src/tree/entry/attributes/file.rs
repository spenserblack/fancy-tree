//! Module for file attributes.
use super::interop::{has_hidden_attribute, is_executable};
use gengo_language::Language;
use std::fs::{File, Metadata};
use std::io::{self, Read};
use std::path::Path;

/// The maximum number of bytes to read from a file to determine its language.
const READ_LIMIT: u16 = 1024 * 16; // 16 KiB

/// Attributes for a file.
pub struct FileAttributes {
    /// Does the file have the hidden attribute set?
    ///
    /// Always `false` on Unix.
    hidden: bool,
    /// The file's language.
    language: Option<Language>,
    /// Is the file an executable?
    executable: bool,
}

impl FileAttributes {
    /// Creates file attributes.
    pub(super) fn new<P>(path: P, file: File, metadata: Metadata) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut contents = file.take(READ_LIMIT.into());
        let mut buf = vec![0; READ_LIMIT.into()];
        let n = contents.read(&mut buf)?;
        buf.truncate(n);
        let language = Language::pick(&path, &buf, READ_LIMIT.into());

        let attributes = FileAttributes {
            hidden: has_hidden_attribute(&metadata),
            language,
            executable: is_executable(path, &metadata),
        };
        Ok(attributes)
    }

    /// Is the file hidden?
    #[inline]
    pub const fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Is the file an executable?
    #[inline]
    pub const fn is_executable(&self) -> bool {
        self.executable
    }

    /// Get the file's language.
    #[inline]
    pub const fn language(&self) -> Option<Language> {
        self.language
    }
}
