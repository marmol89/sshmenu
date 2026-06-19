#!/usr/bin/env bash
set -e

REPO="marmol89/sshmenu"
BINARY="sshmenu"
INSTALL_DIR="${SSHMENU_INSTALL_DIR:-$HOME/.local/bin}"

# Detect platform
detect_platform() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$os" in
        linux) os="linux" ;;
        darwin) os="macos" ;;
        *) echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) echo "Unsupported arch: $arch" >&2; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# Find latest release tag from GitHub
get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep -oE '"tag_name":\s*"[^"]+"' \
            | head -1 \
            | sed 's/.*"v\?\(.*\)"/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep -oE '"tag_name":\s*"[^"]+"' \
            | head -1 \
            | sed 's/.*"v\?\(.*\)"/\1/'
    else
        echo "Error: need curl or wget installed" >&2
        exit 1
    fi
}

download_and_install() {
    local version="$1"
    local platform="$2"
    local archive="${BINARY}-${platform}.tar.gz"
    local url="https://github.com/${REPO}/releases/download/v${version}/${archive}"

    echo "Downloading ${BINARY} v${version} for ${platform}..."
    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "${tmpdir}/${archive}" "$url"
    else
        wget -q -O "${tmpdir}/${archive}" "$url"
    fi

    tar -xzf "${tmpdir}/${archive}" -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    install -m 0755 "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"

    echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        echo ""
        echo "Note: ${INSTALL_DIR} is not in your PATH."
        echo "Add this to your shell config:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

main() {
    local version="${SSHMENU_VERSION:-}"
    local platform
    platform="$(detect_platform)"

    if [[ -z "$version" ]]; then
        version="$(get_latest_version)"
        if [[ -z "$version" ]]; then
            echo "Error: could not determine latest version" >&2
            echo "Set SSHMENU_VERSION env var or install from source" >&2
            exit 1
        fi
    fi

    download_and_install "$version" "$platform"
}

main "$@"