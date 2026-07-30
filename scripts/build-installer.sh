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

# Which commit this build came from.
#
# The version moves when the software changes, not when a build is handed to
# somebody, so several builds share a version. Without this they would share a
# file name too, and a bug report against 0.5.0 could not be matched to the
# code it came from.
#
# Nothing is added at a tag: that build is the release, and the release is the
# version. Everywhere else the commit goes on after a "+", which is build
# metadata, so version ordering ignores it and two builds of one version stay
# equal while remaining tellable apart.
if git describe --exact-match --tags HEAD >/dev/null 2>&1; then
  BUILD=""
else
  commit=$(git rev-parse --short=8 HEAD 2>/dev/null || echo unknown)
  # A build made from edits nobody has committed cannot be matched to
  # anything, and saying so is better than naming a commit it is not.
  if ! git diff --quiet HEAD 2>/dev/null; then
    commit="$commit.dirty"
  fi
  BUILD="g$commit"
fi

if [ -n "$BUILD" ]; then
  FULL_VERSION="$VERSION+$BUILD"
else
  FULL_VERSION="$VERSION"
fi
# Read by build.rs, so the running program and its log say which build they
# are, not only the file it was installed from.
export WIXEN_BUILD="$BUILD"

# The Windows file version field holds four numbers and nothing else, so a
# prerelease has to be encoded rather than carried. Ordinary development
# versions are plain and land on 4000, which is above every prerelease of
# themselves, and that is the order Windows should see.
#
#     0.5.0           ->  0.5.0.4000
#     0.6.0-alpha.1   ->  0.6.0.1001
#     0.6.0-beta.2    ->  0.6.0.2002
#     0.6.0-rc.1      ->  0.6.0.3001
#     0.6.0           ->  0.6.0.4000
#
# The build identifier is not in here. It is metadata rather than a version,
# there is no field for it, and squeezing a commit into a number would only
# produce a number nobody can read back.
IFS='.' read -r major minor patch _ <<<"${VERSION%%-*}"
case "$VERSION" in
  *-alpha.*) stage=1 ;;
  *-beta.*) stage=2 ;;
  *-rc.*) stage=3 ;;
  # Some other prerelease spelling. Below every named stage, which is the
  # safe direction for something we do not recognise.
  *-*) stage=0 ;;
  *) stage=4 ;;
esac
step=$(printf '%s' "$VERSION" | sed -nE 's/.*-(alpha|beta|rc)\.([0-9]+)$/\2/p')
step=${step:-0}
# Each field is capped at 65535 by Windows, and a step of 1000 would read as
# the next stage. Neither has ever been close, and a wrong version number is
# not worth finding out the hard way.
[ "$step" -gt 999 ] && step=999
VERSION_INFO="${major:-0}.${minor:-0}.${patch:-0}.$((stage * 1000 + step))"

find_iscc() {
  if command -v iscc >/dev/null 2>&1; then
    command -v iscc
    return
  fi
  # Inno Setup offers a per-user install as well as a machine-wide one, and
  # neither puts ISCC on the PATH.
  local user_programs="${LOCALAPPDATA:-}"
  user_programs="${user_programs//\\//}"
  for candidate in \
    "${user_programs:+$user_programs/Programs/Inno Setup 6/ISCC.exe}" \
    "/c/Program Files (x86)/Inno Setup 6/ISCC.exe" \
    "/c/Program Files/Inno Setup 6/ISCC.exe"; do
    if [ -n "$candidate" ] && [ -x "$candidate" ]; then
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

echo "== setup executable, version $FULL_VERSION, file version $VERSION_INFO =="
mkdir -p dist
# The doubled slash is not a typo. Git Bash rewrites a leading /D into a
# Windows path; // stops it.
"$ISCC" //DAppVersion="$FULL_VERSION" //DVersionInfo="$VERSION_INFO" installer/Wixen-Mail-Setup.iss

echo
echo "Built dist/Wixen-Mail-Setup-$FULL_VERSION.exe"
