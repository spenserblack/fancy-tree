#!/bin/sh
REPO_DOMAIN=github.com
REPO_OWNER=spenserblack
REPO_NAME=fancy-tree
REPO="$REPO_DOMAIN/$REPO_OWNER/$REPO_NAME"

KERNEL="$(uname -s)"
if [ "$KERNEL" = "Darwin" ]; then
	OS="macOS"
elif [ "$KERNEL" = "Linux" ]; then
	OS="Linux"
else
	echo "Unknown kernel: $KERNEL" >&2
	exit 1
fi

MACHINE="$(uname -m)"
if [ "$MACHINE" = "x86_64" ] || [ "$MACHINE" = "amd64" ]; then
	ARCH="X64"
elif [ "$MACHINE" = "arm64" ]; then
	ARCH="ARM64"
else
	echo "Unknown machine: $MACHINE" >&2
	exit 1
fi

ASSET_NAME="fancy-tree-$OS-$ARCH.tar.gz"
INSTALL_DIR="/usr/local/bin"

echo "Downloading and unpacking to $INSTALL_DIR..."
echo "You may need to activate sudo mode"

curl --proto '=https' -fsSL "https://$REPO/releases/latest/download/$ASSET_NAME" | tar -C "$INSTALL_DIR" -xz
