#!/usr/bin/env bash
set -euo pipefail

# Check if local bin directory exists
[[ ! -d "${HOME}/.local/bin" ]] && {
    echo "Error: ${HOME}/.local/bin not found" >&2
    exit 1
}

# Check if directory is in PATH
[[ ":$PATH:" != *":$HOME/.local/bin:"* ]] && {
    echo "Error: ${HOME}/.local/bin is not in PATH" >&2
    exit 1
}

# Detect system architecture and OS combination
case "$(uname -m)_$(uname -s)" in
"x86_64_Linux") ARCH_OS="x86_64-unknown-linux-gnu" ;;
"x86_64_Darwin") ARCH_OS="x86_64-apple-darwin" ;;
"aarch64_Linux" | "arm64_Linux") ARCH_OS="aarch64-unknown-linux-gnu" ;;
"aarch64_Darwin" | "arm64_Darwin") ARCH_OS="aarch64-apple-darwin" ;;
*)
    echo "Unsupported system" >&2
    exit 1
    ;;
esac

# Set up temporary directory for download (auto-cleaned on exit)
TMP_DIR=$(mktemp -d)
trap 'rm -rf $TMP_DIR' EXIT

# Get latest release, download binary, and install to ~/.local/bin
RELEASE=$(curl -s https://api.github.com/repos/stvnksslr/khelp/releases/latest)
curl -sL "$(echo "$RELEASE" | grep -o "\"browser_download_url\": \"[^\"]*khelp-${ARCH_OS}.tar.gz\"" | cut -d'"' -f4)" | tar xz -C "$TMP_DIR"
mv "$TMP_DIR/khelp" "$HOME/.local/bin/" && chmod +x "$HOME/.local/bin/khelp"
