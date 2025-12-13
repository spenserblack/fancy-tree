# fancy-tree

A `tree` alternative with git support, code language detection, and nerd fonts.

## Features

### Nerd Font icons with language detection

[Nerd Fonts](https://www.nerdfonts.com/) are used for file icons. Each file is analyzed
to determine the appropriate icon and what color the icon should be.

### Git Integration

- Git status is displayed
- Git ignored files are dimmed

## Configuration

*You can edit a config file by calling `fancy-tree --edit-config [CONFIG]`.

The configuration files are Lua modules, which makes them runnable scripts and allow for
complex behavior if wanted. This tool provides a small API under the `fancytree` global
table. Check out [`lua/meta`](./lua/meta/) to see the available utilities.

### `config.lua`

See the [default file][default-main-config] for an example.

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

[default-main-config]: ./src/config/main/config.lua
