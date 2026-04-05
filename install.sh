#!/bin/sh
# Install chesstui on macOS/Linux.
# Usage: curl -fsSL https://raw.githubusercontent.com/millerc13/chesstui/main/install.sh | sh
set -e

REPO="millerc13/chesstui"
INSTALL_DIR="${CHESSTUI_INSTALL_DIR:-$HOME/.local/bin}"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)  TARGET="x86_64-unknown-linux-musl" ;;
            aarch64) TARGET="aarch64-unknown-linux-musl" ;;
            *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin)
        TARGET="universal-apple-darwin"
        ;;
    *)
        echo "Unsupported OS: $OS (use install.ps1 for Windows)"
        exit 1
        ;;
esac

# Get latest release tag
echo "Fetching latest release..."
TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*: "//;s/".*//')
if [ -z "$TAG" ]; then
    echo "Error: could not determine latest release"
    exit 1
fi
echo "Latest release: $TAG"

URL="https://github.com/$REPO/releases/download/$TAG/chesstui-$TAG-$TARGET.tar.gz"

# Download and extract
TMPDIR=$(mktemp -d)
echo "Downloading chesstui $TAG for $TARGET..."
curl -fsSL "$URL" -o "$TMPDIR/chesstui.tar.gz"
tar xzf "$TMPDIR/chesstui.tar.gz" -C "$TMPDIR"

# Install
mkdir -p "$INSTALL_DIR"
BIN=$(find "$TMPDIR" -name chesstui -type f | head -1)
if [ -z "$BIN" ]; then
    echo "Error: binary not found in archive"
    rm -rf "$TMPDIR"
    exit 1
fi
cp "$BIN" "$INSTALL_DIR/chesstui"
chmod +x "$INSTALL_DIR/chesstui"
rm -rf "$TMPDIR"

echo ""
echo "chesstui $TAG installed to $INSTALL_DIR/chesstui"

# Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        echo "Add that line to your ~/.bashrc or ~/.zshrc to make it permanent."
        ;;
esac
