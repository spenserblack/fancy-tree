//! Module for git integration.
use git2::{Repository, StatusOptions};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub mod status;

/// The main struct for git integration.
pub struct Git {
    /// The main repository.
    repository: Repository,
    /// Cached file statuses.
    statuses: HashMap<PathBuf, git2::Status>,
}

impl Git {
    /// Creates a new Git struct.
    ///
    /// If the repository does not exist, this returns `Ok(None)`. Other errors get
    /// passed back to the caller.
    pub fn new<P>(root: P) -> Result<Option<Self>, git2::Error>
    where
        P: AsRef<Path>,
    {
        let result = Repository::discover(root);
        let repo_not_found = result
            .as_ref()
            .is_err_and(|err| matches!(err.code(), git2::ErrorCode::NotFound));
        if repo_not_found {
            Ok(None)
        } else {
            result.and_then(|repository| Self::from_repository(repository).map(Some))
        }
    }

    /// Creates a Git struct from a git2 repository.
    fn from_repository(repository: Repository) -> Result<Self, git2::Error> {
        let statuses = Self::statuses(&repository)?;
        let git = Self {
            repository,
            statuses,
        };
        Ok(git)
    }

    /// Creates a hashmap of paths to statuses for a repository.
    fn statuses(repository: &Repository) -> Result<HashMap<PathBuf, git2::Status>, git2::Error> {
        let mut options = Self::status_options();
        let statuses = repository
            .statuses(Some(&mut options))?
            .iter()
            .map(|entry| {
                let path = entry.path_bytes();
                // SAFETY:
                // - Should always be a path from the local filesystem
                let path = unsafe { OsStr::from_encoded_bytes_unchecked(path) };
                let path = Path::new(path).to_path_buf();
                let status = entry.status();
                (path, status)
            })
            .collect::<HashMap<_, _>>();
        Ok(statuses)
    }

    /// Creates the status options for fetching statuses.
    fn status_options() -> StatusOptions {
        let mut options = StatusOptions::new();
        options
            .include_untracked(true)
            .include_unmodified(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);
        options
    }

    /// Gets the tracked status for a file.
    pub fn tracked_status<P>(&self, path: P) -> Result<Option<status::Tracked>, git2::Error>
    where
        P: AsRef<Path>,
    {
        self.git2_status(path).map(status::Tracked::from_git2)
    }

    /// Gets the untracked status for a file.
    pub fn untracked_status<P>(&self, path: P) -> Result<Option<status::Untracked>, git2::Error>
    where
        P: AsRef<Path>,
    {
        self.git2_status(path).map(status::Untracked::from_git2)
    }

    /// Gets the original gt2 status for a file.
    ///
    /// Path should be relative to the repository's root, and ideally should be as
    /// `path/to/file.ext`. In other words, paths should be as simple as possible, and
    /// not have `./` or `../`
    fn git2_status<P>(&self, path: P) -> Result<git2::Status, git2::Error>
    where
        P: AsRef<Path>,
    {
        // NOTE If the status is not in the cache, then maybe we're looking at an
        //      ignored file or a file that wasn't in the cache due to the status
        //      options set. If that happens we get the status on demand.
        self.cached_git2_status(&path)
            .map(Ok)
            .unwrap_or_else(|| self.on_demand_git2_status(path))
    }

    /// Gets the cached git2 status for a path.
    fn cached_git2_status<P>(&self, path: P) -> Option<git2::Status>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.statuses.get(path).cloned()
    }

    /// Gets the on-demand git2 status for a path.
    fn on_demand_git2_status<P>(&self, path: P) -> Result<git2::Status, git2::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.repository.status_file(path)
    }

    /// Checks if a path is ignored.
    pub fn is_ignored<P>(&self, path: P) -> Result<bool, git2::Error>
    where
        P: AsRef<Path>,
    {
        self.repository.is_path_ignored(path)
    }
}
