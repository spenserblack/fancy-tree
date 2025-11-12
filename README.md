# pretty-ls

A beautiful `ls` alternative with git support, code language detection, and nerd fonts.

## Features

### Language Detection and Syntax Coloring
For code files, `pretty-ls` automatically detects the programming language and colors the filename accordingly, making it easy to identify different file types at a glance.

### Git Integration
- **Git Status Display**: See the git status of files directly in the listing
- **Dimmed Ignored Files**: Files that are ignored by git (via `.gitignore`) are automatically dimmed, helping you focus on tracked files

### Nerd Font Icons
`pretty-ls` uses [Nerd Fonts](https://www.nerdfonts.com/) to display beautiful icons for different file types, providing visual cues that make navigation easier.

### Custom Icon Configuration
You can customize which icons are used for specific files using a `.nf-icons` file. This file uses a format similar to `.gitattributes`, allowing you to specify icon mappings based on file patterns.

#### Example `.nf-icons` file:
```
*.foo 
```

In this example, all files matching the `*.foo` pattern will be displayed with the `` icon.
