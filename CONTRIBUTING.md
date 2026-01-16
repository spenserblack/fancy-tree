# Contributing

This document will guide you through some of the ways that you can contribute to this
project.

## Updating icons

This project takes several steps to determine the icon to use for a file.

1. Look for a known filename
2. Look for a known file extension
3. Try to match the filename to a [glob][glob-crate]
4. Detect the coding language via [gengo][gengo] and use that language's icon

You will find the icon mappings for filenames, file extensions, and [globs][glob-crate]
in [`src/icons/mod.rs`](./src/icons/mod.rs). If you want to update the icon associated
with a coding language, you should instead contribute to [gengo][gengo]. The mappings
in this crate are intended to be *more specific* than a coding language. For example,
a `LICENSE` file is usually plain text. But, rather than use the icon for plain text
files, we know that the *purpose* of a `LICENSE` file is to provide a license, so we use
a license icon instead.

You can pick a new icon from the [Nerd Fonts Cheat Sheet][nf-cheat-sheet]. To help make
the code readable to both contributors *with* Nerd Fonts and contributors *without*
Nerd Fonts, follow this standard:

- The string value should be a [unicode escape][rust-unicode-escape-docs]. This will use
  the hex code at the bottom-right of the icon list item.
- Add a comment with the actual icon.

When adding an icon for a glob, make sure to use the correct syntax for the
[glob crate][glob-crate].

**Match by literal filenames and extensions if you can.** This is much cheaper
computationally than adding a new glob pattern.

[gengo]: https://github.com/spenserblack/gengo
[glob-crate]: https://docs.rs/glob/latest/glob/
[nf-cheat-sheet]: https://www.nerdfonts.com/cheat-sheet
[rust-unicode-escape-docs]: https://doc.rust-lang.org/reference/tokens.html#unicode-escapes
