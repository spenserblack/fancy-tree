//! Provides utilities for file objects.
pub use directory::DirectoryAttributes;
pub use file::FileAttributes;
use std::fs::{self, File, Metadata};
use std::io;
use std::path::Path;
pub use symlink::SymlinkAttributes;

mod directory;
mod file;
mod interop;
mod symlink;

/// Attributes for a tree entry.
pub enum Attributes {
    /// A directory.
    Directory(DirectoryAttributes),
    /// A file.
    File(FileAttributes),
    /// A symlink.
    Symlink(SymlinkAttributes),
}

impl Attributes {
    /// Creates new [`Attributes`].
    pub fn new<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            Ok(Self::new_symlink())
        } else if file_type.is_dir() {
            Ok(Self::new_directory(metadata))
        } else if file_type.is_file() {
            let file = File::open(path)?;
            Self::new_file(path, file, metadata)
        } else {
            // NOTE Just to make all file type checks a bit more explicit
            unreachable!("Must be a symlink, directory, or file")
        }
    }

    /// Creates file attributes.
    #[inline]
    fn new_file<P>(path: P, file: File, metadata: Metadata) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        FileAttributes::new(path, file, metadata).map(Self::File)
    }

    /// Creates directory attributes.
    #[inline]
    fn new_directory(metadata: Metadata) -> Self {
        Self::Directory(DirectoryAttributes::new(metadata))
    }

    /// Creates symlink attributes.
    #[inline]
    fn new_symlink() -> Self {
        Self::Symlink(SymlinkAttributes)
    }

    /// Gets a reference to the file attributes.
    #[inline]
    pub fn file(&self) -> Option<&FileAttributes> {
        if let Self::File(attributes) = self {
            Some(attributes)
        } else {
            None
        }
    }

    /// Gets a reference to the directory attributes.
    #[inline]
    pub fn directory(&self) -> Option<&DirectoryAttributes> {
        if let Self::Directory(attributes) = self {
            Some(attributes)
        } else {
            None
        }
    }

    /// Gets a reference to the symlink attributes.
    #[inline]
    pub fn symlink(&self) -> Option<&SymlinkAttributes> {
        if let Self::Symlink(attributes) = self {
            Some(attributes)
        } else {
            None
        }
    }

    /// Checks if the file is an executable.
    pub fn is_executable(&self) -> bool {
        self.is_file_and(|attributes| attributes.is_executable())
    }

    /// Checks if the attributes mark the file as hidden.
    pub fn is_hidden(&self) -> bool {
        self.is_file_and(|attributes| attributes.is_hidden())
            || self.is_directory_and(|attributes| attributes.is_hidden())
    }

    /// Checks if the attributes are for a file.
    #[inline]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// If the attributes are for a file, calls `f` on the [`FileAttributes`].
    pub fn is_file_and<F>(&self, f: F) -> bool
    where
        F: FnOnce(&FileAttributes) -> bool,
    {
        if let Self::File(attributes) = self {
            f(attributes)
        } else {
            false
        }
    }

    /// Checks if the attributes are for a directory.
    #[inline]
    pub const fn is_directory(&self) -> bool {
        matches!(self, Self::Directory(_))
    }

    /// If the attributes are for a directory, calls `f` on the [`DirectoryAttributes`].
    pub fn is_directory_and<F>(&self, f: F) -> bool
    where
        F: FnOnce(&DirectoryAttributes) -> bool,
    {
        if let Self::Directory(attributes) = self {
            f(attributes)
        } else {
            false
        }
    }

    /// Checks if the attributes are for a symlink.
    #[inline]
    pub const fn is_symlink(&self) -> bool {
        matches!(self, Self::Symlink(_))
    }
}
