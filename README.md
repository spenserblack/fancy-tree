# fancy-tree

[![Crates.io Version](https://img.shields.io/crates/v/fancy-tree)](https://crates.io/crates/fancy-tree)
[![CI](https://github.com/spenserblack/fancy-tree/actions/workflows/ci.yml/badge.svg)](https://github.com/spenserblack/fancy-tree/actions/workflows/ci.yml)

A `tree` alternative with git support, code language detection, and nerd fonts.

## Installation

View [`INSTALL.md`](./INSTALL.md)

## Features

### Nerd Font icons with language detection

[Nerd Fonts](https://www.nerdfonts.com/) are used for file icons. Each file is analyzed to determine the appropriate icon and color.

### Git Integration
- Git status indicators (modified, staged, untracked)
- Git ignored files are visually dimmed

### Highly Configurable
Customize icons, colors, and behavior through Lua configuration files.


## Configuration

*You can edit a config file by calling `fancy-tree --edit-config [CONFIG]`.*

The configuration files are Lua modules, which makes them runnable scripts and allow for
complex behavior if wanted. This tool provides a small API under the `fancytree` global
table. Check out [`lua/meta`](./lua/meta/) to see the available utilities.

### `config.lua`

See the [default file][default-main-config] for an example.

This configures general settings.

### `icons.lua`

See the [default file][default-icon-config] for an example.

This provides a function that takes a filename, file attributes, and the default icon,
 and returns text to use for the icon. Return `nil` to disable the icon.

#### Example

```lua
return function(filename, attributes, default)
  if fancytree.glob_matches("*.config.{js,ts}", filename) then
    return ""
  end
  return default
end
```

### `colors.lua`

See the [default file][default-color-config] for an example.

This provides a function to decide the color for a file's icon, and also functions to
set the colors for git statuses.

[default-main-config]: ./src/config/main/config.lua
[default-color-config]: ./src/config/colors/colors.lua
[default-icon-config]: ./src/config/icons/icons.lua

### 🐛 Troubleshooting

Icons not showing? Ensure you have a Nerd Font installed and selected in your terminal.

Git status missing? Make sure you're in a git repository and have git installed.


### LICENSE

View [`LICENSE`](./INSTALL-APACHE.md)
