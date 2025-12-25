//! Provides the utility for generating a tree.
use crate::color::{Color, ColorChoice};
use crate::config;
use crate::git::status;
use crate::git::{Git, status::Status};
pub use builder::Builder;
pub use charset::Charset;
pub use entry::Entry;
use entry::attributes::{Attributes, FileAttributes};
use owo_colors::AnsiColors;
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::io::{self, Write, stdout};
use std::path::{Path, PathBuf};

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
    /// Provides the characters to print when traversing the directory structure.
    charset: Charset<'charset>,
    /// Controls how the tree colorizes output.
    color_choice: ColorChoice,
    /// Provides configuration choices.
    ///
    /// When this is `None`, default behaviors will be used.
    config: Option<config::Main>,
    /// Provides icon configuration.
    ///
    /// When this is `None`, default behaviors will be used.
    icons: Option<config::Icons>,
    /// Provides color configuration.
    ///
    /// When this is `None`, default behaviors will be used.
    colors: Option<config::Colors>,
}

impl<'git, 'charset, P> Tree<'git, 'charset, P>
where
    P: AsRef<Path>,
{
    /// The default icon to display for files.
    const DEFAULT_FILE_ICON: &'static str = "\u{f0214}"; // 󰈔
    /// The default icon to display when a file is an executable.
    const DEFAULT_EXECUTABLE_ICON: &'static str = "\u{f070e}"; // 󰜎
    /// The default icon to display for directories/folders.
    const DEFAULT_DIRECTORY_ICON: &'static str = "\u{f024b}"; // 󰉋
    /// The default icon to display for symlinks.
    const DEFAULT_SYMLINK_ICON: &'static str = "\u{cf481}"; // 

    /// The icon (padding) to use if there is no icon.
    const EMPTY_ICON: &'static str = " ";

    /// The default color to use for files.
    const DEFAULT_FILE_COLOR: Option<Color> = None;
    /// The default color to use when a file is an executable.
    const DEFAULT_EXECUTABLE_COLOR: Option<Color> = Some(Color::Ansi(AnsiColors::Green));
    /// The default color to use for directories/folders.
    const DEFAULT_DIRECTORY_COLOR: Option<Color> = Some(Color::Ansi(AnsiColors::Blue));
    /// The default color to use for symlinks.
    const DEFAULT_SYMLINK_COLOR: Option<Color> = Some(Color::Ansi(AnsiColors::Cyan));

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

            // NOTE By default entry order is not guaranteed. This explicitly sorts them.
            // TODO Support different sorting algorithms.
            let mut entries = entries.collect::<Vec<_>>();
            entries.sort_by_key(|entry| {
                let path = entry.path();
                path.to_path_buf()
            });
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

        let icon = self.get_icon(entry);
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
            self.color_choice
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
        let is_hidden = entry.is_hidden() || self.is_path_ignored(path);
        self.config
            .as_ref()
            .and_then(|config| config.should_skip(entry, is_hidden).transpose().ok())
            .flatten()
            .unwrap_or(is_hidden)
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

    /// Gets the icon for an entry.
    fn get_icon<P2>(&self, entry: &Entry<P2>) -> String
    where
        P2: AsRef<Path>,
    {
        let default_choice = match entry.attributes() {
            Attributes::Directory(_) => Self::DEFAULT_DIRECTORY_ICON,
            Attributes::File(attributes) => Self::get_file_icon(attributes),
            Attributes::Symlink(_) => Self::DEFAULT_SYMLINK_ICON,
        };
        // TODO Don't panic on get_icon error.
        self.icons
            .as_ref()
            .map(|icons| {
                icons
                    .get_icon(entry, default_choice)
                    .expect("Icon configuration should be valid")
                    .unwrap_or_else(|| String::from(Self::EMPTY_ICON))
            })
            .unwrap_or_else(|| String::from(default_choice))
    }

    /// Gets the icon for a file entry.
    fn get_file_icon(attributes: &FileAttributes) -> &'static str {
        if attributes.is_executable() {
            return Self::DEFAULT_EXECUTABLE_ICON;
        }
        attributes
            .language()
            .and_then(|language| language.nerd_font_glyph())
            .unwrap_or(Self::DEFAULT_FILE_ICON)
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
        // HACK Optimization to avoid calculating colors when they're disabled.
        if self.color_choice.is_off() {
            return write!(writer, "{display}");
        }

        let fg = match entry.attributes() {
            Attributes::Directory(_) => Self::DEFAULT_DIRECTORY_COLOR,
            Attributes::File(attributes) => Self::get_file_color(attributes),
            Attributes::Symlink(_) => Self::DEFAULT_SYMLINK_COLOR,
        };
        let fg = self
            .colors
            .as_ref()
            .and_then(|colors| {
                colors
                    .for_icon(entry, fg)
                    .expect("Colors configuration should be valid")
            })
            .or(fg);

        self.color_choice.write_to(writer, display, fg, None)
    }

    /// Gets the color for a file.
    fn get_file_color(attributes: &FileAttributes) -> Option<Color> {
        attributes
            .language()
            .map(|language| language.rgb())
            .map(|(r, g, b)| Color::Rgb(r, g, b))
            .or_else(|| {
                attributes
                    .is_executable()
                    .then_some(Self::DEFAULT_EXECUTABLE_COLOR)
                    .flatten()
            })
            .or(Self::DEFAULT_FILE_COLOR)
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

    /// Writes a colorized git status.
    fn write_status<S, W, P2>(&self, writer: &mut W, git: &Git, path: P2) -> io::Result<()>
    where
        S: status::StatusGetter + StatusColor,
        W: Write,
        P2: AsRef<Path>,
    {
        const NO_STATUS: &str = " ";

        let status = git.status::<S, _>(path).ok().flatten();
        let color = status.and_then(|status| {
            self.colors.as_ref().map_or_else(
                || S::get_default_color(status),
                |config| {
                    S::get_git_status_color(status, config)
                        .expect("Config should return a valid color")
                },
            )
        });
        let status = status.map(|status| status.as_str()).unwrap_or(NO_STATUS);
        self.color_choice.write_to(writer, status, color, None)
    }

    /// Strips the root path prefix, which is necessary for git tools.
    fn clean_path_for_git2<P2>(&self, path: P2) -> Option<PathBuf>
    where
        P2: AsRef<Path>,
    {
        let git_root = self.git.and_then(|git| git.root_dir())?;

        // HACK Git root seems to have `/` separators, which breaks path cleanup on
        //      Windows. This cleans up the git root so it can be used with
        //      strip_prefix.
        #[cfg(windows)]
        let git_root = git_root
            .canonicalize()
            .expect("Git root should exist and non-final components should be directories");

        let path = path.as_ref();
        let path = path
            .canonicalize()
            .expect("Path should exist and non-final components should be directories");
        let path = path
            .strip_prefix(git_root)
            .expect("Path should have the git root as a prefix");
        Some(path.to_path_buf())
    }
}

/// Private trait to generalize getting the color for a status.
trait StatusColor {
    /// Default color for added status.
    const DEFAULT_ADDED: AnsiColors;
    /// Default color for modified status.
    const DEFAULT_MODIFIED: AnsiColors;
    /// Default color for removed status.
    const DEFAULT_REMOVED: AnsiColors;
    /// Default color for renamed status.
    const DEFAULT_RENAMED: AnsiColors;

    /// Gets the default color for a status.
    fn get_default_color(status: Status) -> Option<Color> {
        use Status::*;

        let default_color = match status {
            Added => Self::DEFAULT_ADDED,
            Modified => Self::DEFAULT_MODIFIED,
            Removed => Self::DEFAULT_REMOVED,
            Renamed => Self::DEFAULT_RENAMED,
        };

        let default_color = Color::Ansi(default_color);
        Some(default_color)
    }

    /// Gets the color for a git status.
    fn get_git_status_color(
        status: Status,
        color_config: &config::Colors,
    ) -> mlua::Result<Option<Color>>;
}

impl StatusColor for status::Tracked {
    const DEFAULT_ADDED: AnsiColors = AnsiColors::Green;
    const DEFAULT_MODIFIED: AnsiColors = AnsiColors::Yellow;
    const DEFAULT_REMOVED: AnsiColors = AnsiColors::Red;
    const DEFAULT_RENAMED: AnsiColors = AnsiColors::Cyan;

    /// Gets the tracked git status color.
    fn get_git_status_color(
        status: Status,
        color_config: &config::Colors,
    ) -> mlua::Result<Option<Color>> {
        let default_choice = Self::get_default_color(status);
        color_config.for_tracked_git_status(status, default_choice)
    }
}

impl StatusColor for status::Untracked {
    const DEFAULT_ADDED: AnsiColors = AnsiColors::BrightGreen;
    const DEFAULT_MODIFIED: AnsiColors = AnsiColors::BrightYellow;
    const DEFAULT_REMOVED: AnsiColors = AnsiColors::BrightRed;
    const DEFAULT_RENAMED: AnsiColors = AnsiColors::BrightCyan;

    /// Gets the untracked git status color.
    fn get_git_status_color(
        status: Status,
        color_config: &config::Colors,
    ) -> mlua::Result<Option<Color>> {
        let default_choice = Self::get_default_color(status);
        color_config.for_untracked_git_status(status, default_choice)
    }
}
