# Install

Installation instructions.

## Download and install from releases

### Unix

```shell
sh <(curl --proto '=https' 'https://raw.githubusercontent.com/spenserblack/fancy-tree/refs/heads/main/scripts/install.sh')
```

### Windows

```powershell
Invoke-WebRequest -UseBasicParsing "https://raw.githubusercontent.com/spenserblack/fancy-tree/refs/heads/main/scripts/install.ps1" | Invoke-Expression
```

### Manual

Visit the [latest release](https://github.com/spenserblack/fancy-tree/releases/latest/) page.

## With `cargo`

```shell
cargo install --locked fancy-tree
```

## With `nix`

```shell
nix --extra-experimental-features nix-command --extra-experimental-features flakes build
```
