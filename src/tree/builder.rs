//! Provides tools for building a [`Tree`].
use super::Tree;
use super::charset::Charset;
use crate::color::ColorChoice;
use crate::config;
use crate::git::Git;
use std::path::Path;

pub struct Builder<'git, 'charset, P: AsRef<Path>> {
    /// The root path for the [`Tree`].
    root: P,
    /// The optional git state.
    git: Option<&'git Git>,
    color_choice: Option<ColorChoice>,
    charset: Option<Charset<'charset>>,
    max_level: Option<usize>,
    config: Option<config::Main>,
    icons: Option<config::Icons>,
    colors: Option<config::Colors>,
}

impl<'git, 'charset, P> Builder<'git, 'charset, P>
where
    P: AsRef<Path>,
{
    /// Creates a new [`Builder`]
    #[inline]
    pub fn new(root: P) -> Self {
        Self {
            root,
            git: None,
            max_level: None,
            charset: None,
            color_choice: None,
            config: None,
            icons: None,
            colors: None,
        }
    }

    /// Adds a git state for the [`Tree`].
    #[inline]
    #[must_use]
    pub fn git(self, git: &'git Git) -> Self {
        Self {
            git: Some(git),
            ..self
        }
    }

    /// Sets the maximum depth level for the [`Tree`].
    #[inline]
    #[must_use]
    pub fn max_level(self, level: usize) -> Self {
        Self {
            max_level: Some(level),
            ..self
        }
    }

    /// Sets the [`Charset`] for the [`Tree`].
    #[inline]
    #[must_use]
    pub fn charset(self, charset: Charset<'charset>) -> Self {
        Self {
            charset: Some(charset),
            ..self
        }
    }

    /// Sets [`ColorChoice`] override for the [`Tree`]. The color choice provided by the
    /// main configuration is used if this isn't set.
    #[inline]
    #[must_use]
    pub fn color_choice(self, color_choice: ColorChoice) -> Self {
        Self {
            color_choice: Some(color_choice),
            ..self
        }
    }

    /// Sets the configuration for the [`Tree`].
    #[inline]
    #[must_use]
    pub fn config(self, config: config::Main) -> Self {
        Self {
            config: Some(config),
            ..self
        }
    }

    /// Sets the icon configuration for the [`Tree`].
    #[inline]
    #[must_use]
    pub fn icons(self, icons: config::Icons) -> Self {
        Self {
            icons: Some(icons),
            ..self
        }
    }

    /// Sets the colors configuration for the [`Tree`].
    #[inline]
    #[must_use]
    pub fn colors(self, colors: config::Colors) -> Self {
        Self {
            colors: Some(colors),
            ..self
        }
    }

    /// Creates the [`Tree`].
    pub fn build(self) -> Tree<'git, 'charset, P> {
        let max_level = self
            .max_level
            .or(self.config.as_ref().and_then(|config| config.level()));
        Tree {
            root: self.root,
            git: self.git,
            max_level,
            charset: self.charset.unwrap_or_default(),
            color_choice: self.color_choice,
            config: self.config.unwrap_or_default(),
            icons: self.icons.unwrap_or_default(),
            colors: self.colors.unwrap_or_default(),
        }
    }
}
