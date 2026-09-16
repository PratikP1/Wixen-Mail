---
name: cutting-a-release
description: How to cut a Wixen Mail release: dispatching the Release workflow, choosing the level, and which levels publish as prereleases. Use when tagging, publishing, or dispatching a release.
---

# Cutting a Wixen Mail release

The versioning policy itself is in `CLAUDE.md` and applies to everyday commits.
This is only the mechanics of the release itself, which matter about once a
cycle.

## Dispatching

Releases are cut deliberately. The Release workflow runs only on manual
dispatch, never on push, and you pick the level when you dispatch it:
`as-is`, `alpha`, `beta`, `rc`, `release`, `patch` or `minor`.

Every level but `as-is` bumps from whatever version it finds, commits that
bump back to `main`, and tags. `as-is` tags the version `Cargo.toml` already
carries and bumps nothing, which is the level an alpha uses: the tree's
version is the next build to go to testers and was moved in the commit that
changed behaviour, so there is nothing left to bump. Until 2026-09-16 this
page listed six levels and every one of them bumped.

**Check `Cargo.toml` before dispatching.** The workflow computes the next
version from the current one, so the level you pick only lands where you expect
if you know where you are starting from. It matters more inside a prerelease:
`alpha` on `1.0.0-alpha.1` publishes `-alpha.2`, and `patch` on it publishes
`1.0.0` as a full release, because cargo-release reads `patch` on a prerelease
as dropping the suffix. The number you mean to publish should already be in
the file, and then `as-is` is the level.

## Which levels publish as what

| Level | Published as |
|---|---|
| `as-is` | GitHub prerelease when the version carries `-alpha`, `-beta` or `-rc`; full release when it does not |
| `alpha`, `beta`, `rc` | GitHub prerelease |
| `patch`, `minor`, `release` | Full release |

Whether a release is a prerelease is decided from the tag's suffix, not from
the level, so `as-is` follows the number it publishes.

Use a full release level only when the version genuinely is what it says. A
full release of something still being handed to testers claims a state the
software is not in. Never dispatch `patch` or `minor` while the version carries
a suffix.

## Before dispatching

- `docs/changelog.md` has an entry under `[Unreleased]` for every user-visible
  change in the cycle, including honest "Known limitations" notes.
- The version in `Cargo.toml` is the one you expect to bump *from*, or, with
  `as-is`, the one you mean to publish.
- The version carries the suffix you mean to publish, and `patch` is not what
  you want on a prerelease.
- `bash scripts/check.sh` passes.
