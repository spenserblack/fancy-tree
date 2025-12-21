//! Module for helpers for git statuses.

use mlua::{IntoLua, Lua};

/// Git statuses (tracked/indexed or untracked/worktree) for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// A new file.
    Added,
    /// A file was changed.
    Modified,
    /// A file was removed.
    Removed,
    /// A file was renamed.
    Renamed,
}

impl Status {
    /// Gets the string representation of a git status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Added => "+",
            Status::Modified => "~",
            Status::Removed => "-",
            Status::Renamed => "R",
        }
    }
}

impl IntoLua for Status {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        use Status::*;

        let s = match self {
            Added => "added",
            Modified => "modified",
            Removed => "removed",
            Renamed => "renamed",
        };

        s.into_lua(lua)
    }
}

/// Trait to generalize getting a git status.
pub trait StatusGetter {
    /// Gets the status from a git2 status.
    fn from_git2(status: git2::Status) -> Option<Status>;
}

/// The tracked git status.
pub struct Tracked;

impl StatusGetter for Tracked {
    /// Gets the index status from the git2 status.
    fn from_git2(status: git2::Status) -> Option<Status> {
        use Status::*;

        let status = if status.is_index_new() {
            Added
        } else if status.is_index_modified() {
            Modified
        } else if status.is_index_deleted() {
            Removed
        } else if status.is_index_renamed() {
            Renamed
        } else {
            return None;
        };

        Some(status)
    }
}

/// The untracked git status.
pub struct Untracked;

impl StatusGetter for Untracked {
    /// Gets the worktree status from the git2 status.
    fn from_git2(status: git2::Status) -> Option<Status> {
        use Status::*;

        let status = if status.is_wt_new() {
            Added
        } else if status.is_wt_modified() {
            Modified
        } else if status.is_wt_deleted() {
            Removed
        } else if status.is_wt_renamed() {
            Renamed
        } else {
            return None;
        };

        Some(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Status::*;
    use git2::Status as Libgit;
    use rstest::rstest;

    #[rstest]
    #[case(Libgit::INDEX_NEW, Some(Added))]
    #[case(Libgit::INDEX_MODIFIED, Some(Modified))]
    #[case(Libgit::INDEX_DELETED, Some(Removed))]
    #[case(Libgit::INDEX_RENAMED, Some(Renamed))]
    #[case(Libgit::WT_NEW, None)]
    fn test_tracked_from_git2(#[case] libgit: Libgit, #[case] expected: Option<Status>) {
        assert_eq!(expected, Tracked::from_git2(libgit));
    }

    #[rstest]
    #[case(Libgit::WT_NEW, Some(Added))]
    #[case(Libgit::WT_MODIFIED, Some(Modified))]
    #[case(Libgit::WT_DELETED, Some(Removed))]
    #[case(Libgit::WT_RENAMED, Some(Renamed))]
    #[case(Libgit::INDEX_NEW, None)]
    fn test_untracked_from_git2(#[case] libgit: Libgit, #[case] expected: Option<Status>) {
        assert_eq!(expected, Untracked::from_git2(libgit));
    }
}
