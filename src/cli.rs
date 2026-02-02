//! CLI utilities.
use crate::color::ColorChoice;
use crate::config::{self, ConfigDir, ConfigFile as _};
use crate::git::Git;
use crate::lua;
use crate::tree;
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;

/// Lists files in a directory.
#[derive(Parser)]
#[deny(missing_docs)]
pub struct Cli {
    /// The path to search in.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Controls colorization.
    #[arg(long = "color")]
    pub color_choice: Option<ColorChoice>,

    /// Go only this many levels deep.
    #[arg(short = 'L', long)]
    pub level: Option<usize>,

    /// Force this tool to have no upper limit for level.
    ///
    /// Useful for overriding a level set by the configuration file.
    #[arg(long, alias = "unset-level", conflicts_with = "level")]
    pub max_level: bool,

    /// Edit the main configuration file and exit.
    #[arg(long, num_args = 0..=1, default_missing_value = "config")]
    pub edit_config: Option<EditConfig>,
}

/// Choices for which config file to edit.
#[derive(ValueEnum, Clone, Copy)]
pub enum EditConfig {
    /// The main configuration file.
    Config,
    /// The custom icon configuration.
    Icons,
    /// The custom colors configuration.
    Colors,
}

impl Cli {
    /// An environment variable the user can set to specify which editor to use.
    const EDITOR_ENV_VAR: &str = "FANCY_TREE_EDITOR";

    /// Runs the CLI.
    pub fn run(&self) -> crate::Result {
        // NOTE Early return for edit mode
        if let Some(edit_config) = self.edit_config {
            return self.edit_file(edit_config);
        }

        self.run_tree()
    }

    /// Runs the main tree functionality.
    fn run_tree(&self) -> crate::Result {
        let git = Git::new(&self.path).expect("Should be able to read the git repository");

        // NOTE The Lua state must live as long as the configuration values.
        let lua_state = {
            let mut builder = lua::state::Builder::new();
            if let Some(ref git) = git {
                builder = builder.with_git(git);
            }
            builder.build().expect("The lua state should be valid")
        };

        // TODO Skip loading the config instead of panicking.
        let config_dir = ConfigDir::new().expect("A config dir should be available");

        let lua_inner = lua_state.to_inner();
        let config = config_dir
            .load_main(lua_inner)
            .expect("The configuration should be valid");
        let icons = config_dir
            .load_icons(lua_inner)
            .expect("The icon configuration should be valid");
        let colors = config_dir
            .load_colors(lua_inner)
            .expect("The color configuration should be valid");

        let mut builder = tree::Builder::new(&self.path);

        // NOTE Apply configuration overrides from CLI.
        if let Some(color_choice) = self.color_choice {
            builder = builder.color_choice(color_choice);
        }

        // NOTE Apply configurations if they exist
        if let Some(config) = config {
            builder = builder.config(config);
        }
        if let Some(icons) = icons {
            builder = builder.icons(icons);
        }
        if let Some(colors) = colors {
            builder = builder.colors(colors);
        }

        if let Some(ref git) = git {
            builder = builder.git(git);
        }

        if let Some(level) = self.level {
            builder = builder.max_level(level);
        } else if self.max_level {
            builder = builder.unset_level();
        }

        let tree = builder.build();

        lua_state.in_git_scope(|| tree.write_to_stdout().map_err(mlua::Error::external))?;

        Ok(())
    }

    /// Opens an editor for the file the user specified, creating the config directory
    /// if needed.
    fn edit_file(&self, edit_config: EditConfig) -> crate::Result {
        let config_dir = ConfigDir::new()?;
        fs::create_dir_all(config_dir.path())?;

        let (file_path, default_contents) = match edit_config {
            EditConfig::Config => (config_dir.main_path(), config::Main::DEFAULT_MODULE),
            EditConfig::Icons => (config_dir.icons_path(), config::Icons::DEFAULT_MODULE),
            EditConfig::Colors => (config_dir.colors_path(), config::Colors::DEFAULT_MODULE),
        };

        // NOTE If we can't check if it exists, we'll be safe and skip overwriting it.
        if !file_path.try_exists().unwrap_or(false) {
            // NOTE Ignore error, because editing the file is a higher priority than
            //      writing to it.
            let _ = fs::write(&file_path, default_contents);
        }

        println!("Opening `{}`", file_path.display());

        let finder = find_editor::Finder::with_extra_environment_variables([Self::EDITOR_ENV_VAR]);
        /// Should the program wait for the editor to close before continuing?
        const WAIT: bool = true;
        finder.open_editor(file_path, WAIT)?;

        Ok(())
    }
}

// Runs the CLI. Can exit early without returning an error. For example, this will exit
// early if the user passes `-h` as CLI argument.
pub fn run() -> crate::Result {
    Cli::parse().run()
}
