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
`patch`, `minor`, `alpha`, `beta`, `rc`, or `release`.

It bumps from whatever version it finds, commits that bump back to `main`, and
tags.

**Check `Cargo.toml` before dispatching.** The workflow computes the next
version from the current one, so the level you pick only lands where you expect
if you know where you are starting from.

## Which levels publish as what

| Level | Published as |
|---|---|
| `alpha`, `beta`, `rc` | GitHub prerelease |
| `patch`, `minor`, `release` | Full release |

Use a full release level only when the version genuinely is what it says. A
full release of something still being handed to testers claims a state the
software is not in.

## Before dispatching

- `docs/changelog.md` has an entry under `[Unreleased]` for every user-visible
  change in the cycle, including honest "Known limitations" notes.
- The version in `Cargo.toml` is the one you expect to bump *from*.
- `bash scripts/check.sh` passes.
