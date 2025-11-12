# pretty-ls

An `ls` alternative with git support, code language detection, and nerd fonts.

## Features

### Language Detection and Syntax Coloring

For code files, the tool detects the programming language and colors the filename accordingly.

### Git Integration

- Git status is displayed
- Git ignored files are dimmed

### Nerd Font Icons

[Nerd Fonts](https://www.nerdfonts.com/) are used for file icons.

### Custom Icon Configuration

A `.nf-icons` file can be used to customize which icons are used for specific files. This file uses a format similar to `.gitattributes`.

#### Example `.nf-icons` file

```
*.foo 
```

Files matching `*.foo` will be displayed with the `` icon.
