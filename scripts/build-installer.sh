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

# How far back the version being stamped was last moved.
#
# This project's rule is that a new feature, a schema change or a behaviour
# change moves the version in the commit that makes it. That rule has lapsed
# three times, twice badly enough to need a catch-up bump: 0.25.0 after one
# feature landed on top of 0.24.0, and 0.33.0 after sixteen commits added
# filing mail into a folder from a rule, moved the compose toolbar, gave Sent
# and Drafts their own columns and built the Account Manager's Sign In Again.
#
# No check can tell a behaviour change from a refactor, so this does not try
# to refuse anything. It says the number out loud at the one moment the drift
# starts to matter, which is somebody building a file to give to another
# person. Several builds sharing a version is normal and expected; sixteen
# commits of feature work sharing one is the thing worth seeing.
#
# Since 2026-09-17 the number is also part of what the build carries, which
# is why it is computed here, before the build identifier below: two builds
# of one version are put in order by it, so it has to exist first.
#
# `git log -S` searches history for the commit that changed how many times
# this exact line appears, which is the commit that set the current number.
VERSION_SET_AT=$(git log --format=%H -S"version = \"$VERSION\"" -- Cargo.toml 2>/dev/null | head -1)
if [ -z "$VERSION_SET_AT" ]; then
  # A build that cannot say its order is refused rather than made. The count
  # exists so that two builds of one version can be put in order by reading
  # them (Pratik's decision of 2026-09-17), and a build stamped without it
  # would be exactly the build nobody can place. Until 2026-09-17 this printed
  # "unknown" and built anyway. A shallow clone has no history to search, and
  # saying "0 commits ago" would read as "just bumped", which is the most
  # reassuring answer available and the one thing this does not know.
  echo "This clone does not hold the commit that set $VERSION, so the build cannot say how many commits it is past it." >&2
  echo "A shallow clone has no history to search; fetch the whole history and build again." >&2
  exit 1
fi
# The count itself. Nothing catches a failure here: under `set -e` a count
# that cannot be taken stops the script, which is the refusal above by another
# route, rather than reaching the arithmetic below as a word.
LAG=$(git rev-list --count "$VERSION_SET_AT..HEAD")

# Which commit this build came from, and how far past its version it is.
#
# The version moves when the software changes, not when a build is handed to
# somebody, so several builds share a version. Without this they would share a
# file name too, and a bug report against 0.5.0 could not be matched to the
# code it came from. Since 2026-09-17 the number of commits since the version
# was set goes in front of the commit, so two builds of one version can be put
# in order by reading them: 1.0.0-alpha.1+42.g59c5b6a4 is later than
# 1.0.0-alpha.1+40.gabcdef12, where a hash alone orders nothing. The same
# number goes into the file version below, so Apps and Features orders the
# builds the same way.
#
# Nothing is added at a tag: that build is the release, and the release is the
# version. Everywhere else the counter and the commit go on after a "+", which
# is build metadata, so version ordering ignores it and two builds of one
# version stay equal while remaining tellable apart. The file version keeps
# the counter at a tag even so, for the reason given at the table below.
if git describe --exact-match --tags HEAD >/dev/null 2>&1; then
  BUILD=""
else
  # No fallback to the word unknown here, since 2026-09-17: a clone where
  # HEAD cannot be named has already been refused above, where the version's
  # commit could not be found, so a failure here stops the script under
  # `set -e` rather than stamping a word into the file name.
  commit=$(git rev-parse --short=8 HEAD)
  # A build made from edits nobody has committed cannot be matched to
  # anything, and saying so is better than naming a commit it is not.
  if ! git diff --quiet HEAD 2>/dev/null; then
    commit="$commit.dirty"
  fi
  BUILD="$LAG.g$commit"
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
# prerelease and the build counter have to be encoded rather than carried.
# The fourth field weighs the stage, the step and the counter so that every
# later build reads higher than every earlier one, which is the order Windows
# should see when one build is installed over another:
#
#     1.0.0-alpha.1+70.g7d57cd49   ->  1.0.0.14070
#     1.0.0-alpha.1+88.g59c5b6a4   ->  1.0.0.14088
#     1.0.0-alpha.2 (just set)     ->  1.0.0.15000
#     1.0.0-beta.1                 ->  1.0.0.27000
#     1.0.0-rc.1                   ->  1.0.0.40000
#     1.0.0 at its tag             ->  1.0.0.52000
#     1.0.0+3.gabcdef12            ->  1.0.0.52003
#     0.5.0 (plain, nothing since) ->  0.5.0.52000
#
# Why these weights. Windows allows 65535 in a field, and five stages (an
# unrecognised prerelease, alpha, beta, rc, plain) under that leave 13107
# each, so the stage weighs 13000. The step weighs a thousand so that a
# counter under a thousand can never read as the next step, and with the
# step held to 12 a step can never read as the next stage: 12 * 1000 + 999 is
# 12999, under 13000. The largest value is then 4 * 13000 + 12 * 1000 + 999,
# which is 64999. A weight of 10000 for the stage, which was considered, puts
# rc.11 at 41000 above a release at 40000, so the order it exists for breaks
# at the eleventh release candidate; the weight here is what keeps it.
#
# The caps are where the arithmetic would otherwise lie, and each says so out
# loud when it bites rather than producing a quietly wrong number. The
# version string still carries the true number in both cases.
#
# A build at a tag omits the counter from the string, as above, and keeps it
# here: the release then sits above the tester builds made before it and
# below those after. A counter of zero there would order the release below
# every build that staged it, and Apps and Features would read installing the
# release over the last release candidate as a downgrade.
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
counter=$LAG
if [ "$step" -gt 12 ]; then
  echo "== the step is $step and the file version holds it to 12 =="
  echo "   Apps and Features cannot order this build against another past the"
  echo "   cap; the version string still carries the true number."
  step=12
fi
if [ "$counter" -gt 999 ]; then
  echo "== the build counter is $LAG and the file version holds it to 999 =="
  echo "   Apps and Features cannot order this build against another past the"
  echo "   cap; the version string still carries the true number."
  counter=999
fi
VERSION_INFO="${major:-0}.${minor:-0}.${patch:-0}.$((stage * 13000 + step * 1000 + counter))"

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

case "$LAG" in
  0)
    echo "== $VERSION was set in this very commit =="
    ;;
  1)
    echo "== $VERSION was set 1 commit ago =="
    ;;
  *)
    echo "== $VERSION was set $LAG commits ago =="
    echo "   Several builds sharing a version is normal. If any of those $LAG"
    echo "   added a feature or changed behaviour, the version should have"
    echo "   moved with it. See the versioning rules in CLAUDE.md."
    ;;
esac
echo

echo "== release build =="
cargo build --release

# The Windows Search handler is its own crate with its own target folder, so
# `cargo build` above does not touch it. The installer names both files it
# builds and refuses to compile without them, which is the point: a release that
# quietly shipped without the search handler would look complete and the feature
# would simply not be there.
echo "== search handler =="
(cd search-handler && cargo build --release)

echo "== setup executable, version $FULL_VERSION, file version $VERSION_INFO =="
mkdir -p dist
# The doubled slash is not a typo. Git Bash rewrites a leading /D into a
# Windows path; // stops it.
"$ISCC" //DAppVersion="$FULL_VERSION" //DVersionInfo="$VERSION_INFO" installer/Wixen-Mail-Setup.iss

echo
echo "Built dist/Wixen-Mail-Setup-$FULL_VERSION.exe"
