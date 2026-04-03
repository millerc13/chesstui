#!/usr/bin/env bash
#
# chesstui installer
# Usage:  curl -fsSL https://raw.githubusercontent.com/cjmiller/chesstui/main/scripts/install.sh | bash
#         curl -fsSL ... | bash -s -- --version v0.2.0   (pin a version)
#
set -euo pipefail

REPO="cjmiller/chesstui"
BINARY="chesstui"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
info()  { printf "\033[1;34m=>\033[0m %s\n" "$*"; }
error() { printf "\033[1;31mERROR:\033[0m %s\n" "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" > /dev/null 2>&1 || error "need '$1' (command not found)"
}

# ---------------------------------------------------------------------------
# Detect OS and architecture
# ---------------------------------------------------------------------------
detect_target() {
  local os arch target

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64)  target="x86_64-unknown-linux-musl" ;;
        aarch64) target="aarch64-unknown-linux-musl" ;;
        arm64)   target="aarch64-unknown-linux-musl" ;;
        *)       error "Unsupported Linux architecture: $arch" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64)  target="x86_64-apple-darwin" ;;
        arm64)   target="aarch64-apple-darwin" ;;
        *)       error "Unsupported macOS architecture: $arch" ;;
      esac
      ;;
    *)
      error "Unsupported OS: $os (use the PowerShell installer on Windows)"
      ;;
  esac

  echo "$target"
}

# ---------------------------------------------------------------------------
# Resolve version (latest or pinned)
# ---------------------------------------------------------------------------
resolve_version() {
  local version="${1:-}"

  if [ -n "$version" ]; then
    echo "$version"
    return
  fi

  need_cmd curl
  need_cmd grep
  need_cmd cut

  local latest
  latest="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | cut -d'"' -f4)"

  [ -n "$latest" ] || error "Could not determine latest version"
  echo "$latest"
}

# ---------------------------------------------------------------------------
# Download, verify, install
# ---------------------------------------------------------------------------
main() {
  local version=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --version) version="$2"; shift 2 ;;
      *) error "Unknown option: $1" ;;
    esac
  done

  need_cmd curl
  need_cmd tar

  local target version_tag archive_name url
  target="$(detect_target)"
  version_tag="$(resolve_version "$version")"
  archive_name="${BINARY}-${version_tag}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version_tag}/${archive_name}"
  checksums_url="https://github.com/${REPO}/releases/download/${version_tag}/SHA256SUMS.txt"

  info "Installing ${BINARY} ${version_tag} for ${target}"

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  info "Downloading ${url}"
  curl -fsSL "$url" -o "${tmpdir}/${archive_name}"

  # Verify checksum if sha256sum is available
  if command -v sha256sum > /dev/null 2>&1; then
    info "Verifying checksum..."
    curl -fsSL "$checksums_url" -o "${tmpdir}/SHA256SUMS.txt"
    cd "$tmpdir"
    grep "$archive_name" SHA256SUMS.txt | sha256sum -c - || error "Checksum verification failed"
    cd -
  elif command -v shasum > /dev/null 2>&1; then
    info "Verifying checksum..."
    curl -fsSL "$checksums_url" -o "${tmpdir}/SHA256SUMS.txt"
    cd "$tmpdir"
    grep "$archive_name" SHA256SUMS.txt | shasum -a 256 -c - || error "Checksum verification failed"
    cd -
  else
    info "Skipping checksum verification (no sha256sum or shasum found)"
  fi

  info "Extracting..."
  tar xzf "${tmpdir}/${archive_name}" -C "${tmpdir}"

  info "Installing to ${INSTALL_DIR}/${BINARY}"
  if [ -w "$INSTALL_DIR" ]; then
    cp "${tmpdir}/${BINARY}-${version_tag}-${target}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
  else
    sudo cp "${tmpdir}/${BINARY}-${version_tag}-${target}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY}"
  fi

  info "Installed ${BINARY} ${version_tag} to ${INSTALL_DIR}/${BINARY}"
  info "Run '${BINARY} --help' to get started"
}

main "$@"
