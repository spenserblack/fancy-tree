//! Provides the utility for generating a tree.
use crate::color::{Color, ColorChoice};
use crate::config;
use crate::git::status::StatusGetter;
use crate::git::{
    Git,
    status::{self, Status},
};
pub use builder::Builder;
pub use charset::Charset;
pub use entry::Entry;
use owo_colors::AnsiColors;
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::io::{self, Write, stdout};
use std::path::{self, Path, PathBuf};

mod builder;
mod charset;
pub mod entry;

/// Generates a tree.
pub struct Tree<'git, 'charset, P: AsRef<Path>> {
    /// The root path to start from.
    root: P,
    /// The optional git state of the directory.
    git: Option<&'git Git>,
    /// The maximum depth level to display.
    max_level: Option<usize>,
    /// Overrides the configured color choice (e.g. if specified in the CLI).
    color_choice: Option<ColorChoice>,
    /// Provides the characters to print when traversing the directory structure.
    charset: Charset<'charset>,
    /// Provides configuration choices.
    ///
    /// When this is `None`, default behaviors will be used.
    config: config::Main,
    /// Provides icon configuration.
    icons: config::Icons,
    /// Provides color configuration.
    colors: config::Colors,
}

impl<'git, 'charset, P> Tree<'git, 'charset, P>
where
    P: AsRef<Path>,
{
    /// Writes the tree to stdout.
    #[inline]
    pub fn write_to_stdout(&self) -> crate::Result<()>
    where
        P: AsRef<Path>,
    {
        let mut stdout = stdout();
        self.write(&mut stdout)?;
        Ok(())
    }

    /// Writes to the writer.
    pub fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        let Ok(entry) = Entry::new(&self.root) else {
            // HACK We can't read the first entry for some reason, so we'll just print
            //      it and exit.
            let path = self.root.as_ref();
            Self::write_path(writer, path)?;
            return writeln!(writer);
        };
        self.write_depth(writer, entry, 0)?;
        writer.flush()
    }

    /// Writes the tree at a certain depth to the writer.
    fn write_depth<W, P2>(&self, writer: &mut W, entry: Entry<P2>, depth: usize) -> io::Result<()>
    where
        W: Write,
        P2: AsRef<Path>,
    {
        let path = entry.path();

        // NOTE For the top level, we always print the full path the user specified.
        self.write_entry(writer, &entry, depth == 0)?;

        writeln!(writer)?;
        if !path.is_dir() {
            return Ok(());
        }

        // NOTE We'll just skip file read errors to continue printing the rest of the
        //      tree.
        let entries = match path.read_dir() {
            Ok(entries) => entries.filter_map(Result::ok),
            Err(_) => return Ok(()),
        };
        let entries = {
            let entries = entries.map(|entry| entry.path()).map(Entry::new);
            // NOTE If we can't read a directory entry, then we'll just ignore it so that
            //      we don't stop early.
            let entries = entries.filter_map(Result::ok);

            // NOTE If the config exists and it successfully detects if a file should
            //      be skipped, use that value. Otherwise, use default behavior.
            let entries = entries.filter(|entry| !self.should_skip_entry(entry));

            let mut entries = entries.collect::<Vec<_>>();
            entries.sort_by(|left, right| self.config.cmp(left.path(), right.path()));
            entries
        };
        if self.max_level.map(|max| depth >= max).unwrap_or(false) {
            return Ok(());
        }

        for entry in entries {
            self.write_indentation(writer, depth)?;
            write!(writer, "{}", self.charset.depth)?;
            self.write_depth(writer, entry, depth + 1)?;
        }

        Ok(())
    }

    /// Writes an entry.
    fn write_entry<W, P2>(&self, writer: &mut W, entry: &Entry<P2>, is_top: bool) -> io::Result<()>
    where
        W: Write,
        P2: AsRef<Path>,
    {
        let path = entry.path();
        self.write_statuses(writer, path)?;

        let icon = self.icons.get_icon(entry);
        self.write_colorized_for_entry(entry, writer, icon)?;
        // NOTE Padding for the icons
        write!(writer, " ")?;

        // HACK is_path_ignored tries to strip the prefix, which we never want to do at
        //      the top when the path is *only* the prefix. In fact, we don't want to
        //      check ignore status here at all since the current implementation breaks
        //      for paths that contain the directory `.`, it seems. Also, the top
        //      should always be a directory, and the current implementation only seems
        //      to work for files.
        let is_ignored = !is_top && self.is_path_ignored(path);

        let path = if is_top {
            path.as_os_str()
        } else {
            // NOTE The only time the path shouldn't have a file name is at the top
            //      level, which could be a path like "." or "..". At the top level
            //      call, `full_name` should always receive `true`.
            path.file_name()
                .expect("A directory entry should always have a file name")
        };

        if !is_ignored {
            Self::write_path(writer, path)
        } else {
            const TEXT_COLOR: Option<Color> = Some(Color::Ansi(AnsiColors::Black));
            self.color_choice()
                .write_to(writer, path.display(), TEXT_COLOR, None)
        }
    }

    /// Writes a path's name.
    fn write_path<W, P2>(writer: &mut W, path: P2) -> io::Result<()>
    where
        W: Write,
        P2: AsRef<Path>,
    {
        let path = path.as_ref();
        writer.write_all(path.as_os_str().as_encoded_bytes())
    }

    /// Writes indentation.
    fn write_indentation<W>(&self, writer: &mut W, level: usize) -> io::Result<()>
    where
        W: Write,
    {
        for _ in 0..level {
            write!(writer, "{}", self.charset.breadth)?;
        }
        Ok(())
    }

    /// Checks if an entry should be skipped.
    ///
    /// If the config exists, the config has a `skip` function, *and* that function
    /// successfully returns a boolean value, then that value will be used. Otherwise,
    /// it will just skip all hidden files.
    fn should_skip_entry<P2>(&self, entry: &Entry<P2>) -> bool
    where
        P2: AsRef<Path>,
    {
        let path = entry.path();
        self.config
            .should_skip(entry, || self.is_path_ignored(path))
    }

    /// Checks if a path is ignored.
    fn is_path_ignored<P2>(&self, path: P2) -> bool
    where
        P2: AsRef<Path>,
    {
        self.git
            .and_then(|git| {
                // HACK This function doesn't expect a `./` prefix. It seems to return
                //      `true` when it's present???
                let path = self
                    .clean_path_for_git2(path)
                    .expect("Should be able to resolve path relative to git root");
                git.is_ignored(path).ok()
            })
            .unwrap_or(false)
    }

    /// Writes the text in a colored style.
    fn write_colorized_for_entry<W, D, P2>(
        &self,
        entry: &Entry<P2>,
        writer: &mut W,
        display: D,
    ) -> io::Result<()>
    where
        W: Write,
        D: Display + OwoColorize,
        P2: AsRef<Path>,
    {
        let color_choice = self.color_choice();

        // HACK Optimization to avoid calculating colors when they're disabled.
        if color_choice.is_off() {
            return write!(writer, "{display}");
        }

        let fg = self.colors.for_icon(entry);
        color_choice.write_to(writer, display, fg, None)
    }

    /// Writes colorized git statuses.
    fn write_statuses<W>(&self, writer: &mut W, path: &Path) -> io::Result<()>
    where
        W: Write,
    {
        let Some(git) = self.git else { return Ok(()) };

        // HACK cached status keys don't have a ./ prefix and git2 apparently doesn't expect it.
        let path = self
            .clean_path_for_git2(path)
            .expect("Should be able to resolve path relative to git root");

        self.write_status::<status::Untracked, _, _>(writer, git, &path)?;
        self.write_status::<status::Tracked, _, _>(writer, git, path)?;
        Ok(())
    }

    /// Writes a colorized untracked (worktree) git status.
    fn write_status<S, W, P2>(&self, writer: &mut W, git: &Git, path: P2) -> io::Result<()>
    where
        S: StatusGetter + ColoredStatus,
        W: Write,
        P2: AsRef<Path>,
    {
        const NO_STATUS: &str = " ";

        let status = git.status::<S, _>(path).ok().flatten();
        let color = status.and_then(|status| S::get_color(&self.colors, status));
        let status = status.map(|status| status.as_str()).unwrap_or(NO_STATUS);
        self.color_choice().write_to(writer, status, color, None)
    }

    /// Strips the root path prefix, which is necessary for git tools.
    fn clean_path_for_git2<P2>(&self, path: P2) -> Option<PathBuf>
    where
        P2: AsRef<Path>,
    {
        let git_root = self.git.and_then(|git| git.root_dir())?;
        clean_path_for_git2(git_root, path)
    }

    /// Gets the color choice to use.
    fn color_choice(&self) -> ColorChoice {
        self.color_choice.unwrap_or(self.config.color_choice())
    }
}

/// Private trait to generalize writing statuses.
trait ColoredStatus {
    /// Gets the color for the status.
    fn get_color(config: &config::Colors, status: Status) -> Option<Color>;
}

impl ColoredStatus for status::Untracked {
    #[inline]
    fn get_color(config: &config::Colors, status: Status) -> Option<Color> {
        config.for_untracked_git_status(status)
    }
}

impl ColoredStatus for status::Tracked {
    #[inline]
    fn get_color(config: &config::Colors, status: Status) -> Option<Color> {
        config.for_tracked_git_status(status)
    }
}

/// Helper for cleaning up a file path so that it can be used with the opened
/// [`git2::Repository`].
fn clean_path_for_git2<P1, P2>(git_root: P1, path: P2) -> Option<PathBuf>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let git_root = git_root.as_ref();

    // HACK Git root seems to have `/` separators, which breaks path cleanup on
    //      Windows. This cleans up the git root so it can be used with
    //      strip_prefix.
    #[cfg(windows)]
    let git_root = git_root
        .canonicalize()
        .expect("Git root should exist and non-final components should be directories");

    let path = path.as_ref();
    let path = path::absolute(path)
        .expect("Path should be non-empty and should be able to get the current directory");
    let path = path
        .strip_prefix(git_root)
        .expect("Path should have the git root as a prefix");
    Some(path.to_path_buf())
}
