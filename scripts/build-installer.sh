#!/usr/bin/env bash
# Build the Windows setup executable.
#
# The same script runs locally and in CI, so the file somebody downloads is
# built the way the one you tested was built. Calling ISCC by hand is not the
# same thing: the version is read from Cargo.toml here, and the .iss refuses to
# compile without it rather than shipping a number somebody forgot to change.
set -euo pipefail

cd "$(dirname "$0")/.."

VERSION=$(grep -m1 -E '^version = "' Cargo.toml | sed -E 's/^version = "(.*)"/\1/')
if [ -z "$VERSION" ]; then
  echo "Could not read the version from Cargo.toml" >&2
  exit 1
fi

# The Windows file version field holds four numbers and nothing else, so the
# prerelease part is dropped for that one field. Every version a person reads
# is the full one.
IFS='.' read -r major minor patch _ <<<"${VERSION%%-*}"
VERSION_INFO="${major:-0}.${minor:-0}.${patch:-0}.0"

find_iscc() {
  if command -v iscc >/dev/null 2>&1; then
    command -v iscc
    return
  fi
  for candidate in \
    "/c/Program Files (x86)/Inno Setup 6/ISCC.exe" \
    "/c/Program Files/Inno Setup 6/ISCC.exe"; do
    if [ -x "$candidate" ]; then
      echo "$candidate"
      return
    fi
  done
}

# Checked before the build rather than after it, so a missing tool costs a
# second instead of a full release compile.
ISCC=$(find_iscc)
if [ -z "$ISCC" ]; then
  echo "Building $VERSION needs Inno Setup 6, which is not installed." >&2
  echo "Install it with:" >&2
  echo >&2
  echo "    winget install JRSoftware.InnoSetup" >&2
  exit 1
fi

echo "== release build =="
cargo build --release

echo "== setup executable, version $VERSION =="
mkdir -p dist
# The doubled slash is not a typo. Git Bash rewrites a leading /D into a
# Windows path; // stops it.
"$ISCC" //DAppVersion="$VERSION" //DVersionInfo="$VERSION_INFO" installer/Wixen-Mail-Setup.iss

echo
echo "Built dist/Wixen-Mail-Setup-$VERSION.exe"
