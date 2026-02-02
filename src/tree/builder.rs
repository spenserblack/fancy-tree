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
    /// Override the level limit that may be set by the configuration.
    unset_level: bool,
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
            unset_level: false,
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

    /// Unsets the maximum depth level for the [`Tree`], returning to the default
    /// behavior of searching infinitely deep.
    ///
    /// This helps override a maximum level that may have been set by the configuration.
    #[inline]
    #[must_use]
    pub fn unset_level(self) -> Self {
        Self {
            unset_level: true,
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
    ///
    /// # Panics
    ///
    /// - Panics if `max_level` and `unset_level` were both called.
    pub fn build(self) -> Tree<'git, 'charset, P> {
        assert!(
            !(self.unset_level && self.max_level.is_some()),
            "max_level cannot be set when unset_level is true"
        );
        let max_level = if self.unset_level {
            None
        } else {
            self.max_level
                .or(self.config.as_ref().and_then(|config| config.level()))
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_cannot_build_unset_level_with_max_level() {
        Builder::new(".").max_level(1).unset_level().build();
    }
}
