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

## From source

Build the executable from source code and then copy it to a place in `PATH`.

### With `cargo`

```shell
cargo build --release
```

### With Nix

#### The classic way

```shell
nix-build
```

#### The Nix Flakes way

```shell
nix --experimental-features nix-command --extra-experimental-features flakes build
```
