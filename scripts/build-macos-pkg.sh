#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BINARY_NAME="${BINARY_NAME:-wetwin}"
PACKAGE_IDENTIFIER="${PACKAGE_IDENTIFIER:-io.github.life2you.wetwin}"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local/bin}"
VERSION=""
BINARY_PATH=""
OUTPUT_PATH=""

require_command() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Required command not found: $cmd" >&2
    exit 1
  fi
}

require_command pkgbuild
require_command install

show_help() {
  cat <<EOF
Usage: ./scripts/build-macos-pkg.sh --binary-path PATH --output PATH [options]

Options:
  --binary-path PATH      Path to the built wetwin binary.
  --output PATH           Output .pkg path.
  --version VERSION       Package version. Defaults to Cargo.toml version.
  --install-prefix PATH   Install prefix inside the package. Default: $INSTALL_PREFIX
  --identifier ID         Package identifier. Default: $PACKAGE_IDENTIFIER
  --help                  Show this help message.

Example:
  ./scripts/build-macos-pkg.sh \\
    --binary-path target/aarch64-apple-darwin/release/wetwin \\
    --output release-assets/wetwin-aarch64.pkg \\
    --version 0.1.1
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary-path)
      BINARY_PATH="${2:-}"
      shift 2
      ;;
    --output)
      OUTPUT_PATH="${2:-}"
      shift 2
      ;;
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --install-prefix)
      INSTALL_PREFIX="${2:-}"
      shift 2
      ;;
    --identifier)
      PACKAGE_IDENTIFIER="${2:-}"
      shift 2
      ;;
    --help)
      show_help
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      show_help >&2
      exit 1
      ;;
  esac
done

if [[ -z "$BINARY_PATH" || -z "$OUTPUT_PATH" ]]; then
  echo "--binary-path and --output are required." >&2
  show_help >&2
  exit 1
fi

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "Binary not found: $BINARY_PATH" >&2
  exit 1
fi

VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' "$REPO_ROOT/Cargo.toml" | head -n1)}"
if [[ -z "$VERSION" ]]; then
  echo "Failed to detect version from Cargo.toml" >&2
  exit 1
fi

STAGE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/wetwin-pkg.XXXXXX")"
cleanup() {
  rm -rf "$STAGE_DIR"
}
trap cleanup EXIT

ROOT_DIR="$STAGE_DIR/root"
TARGET_DIR="$ROOT_DIR$INSTALL_PREFIX"

mkdir -p "$TARGET_DIR"
install -m 755 "$BINARY_PATH" "$TARGET_DIR/$BINARY_NAME"
mkdir -p "$(dirname "$OUTPUT_PATH")"

pkgbuild \
  --root "$ROOT_DIR" \
  --identifier "$PACKAGE_IDENTIFIER" \
  --version "$VERSION" \
  --install-location "/" \
  "$OUTPUT_PATH"

echo "Built package: $OUTPUT_PATH"
echo "Version:       $VERSION"
echo "Install path:  $INSTALL_PREFIX/$BINARY_NAME"
