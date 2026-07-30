# Beta Release and Setup Executables

First beta tag target: `v0.1.0-beta.1`

## Tell testers about the SmartScreen warning

Every alpha and beta build is unsigned, so everyone who downloads one meets
"Windows protected your PC" before they meet Wixen Mail. Say so in the message
that invites them, not only in the documentation, because somebody who hits an
unexplained security warning on the first thing you asked them to do reasonably
stops there.

The wording to give them:

> Windows will warn you that it does not recognise this program. It is not
> signed yet. Choose **More info**, then **Run anyway**. The button you land on
> first is **Don't run**, so do not just press Enter.

That last sentence is the one that matters for screen reader users: the Run
button only exists after the More info link is activated, and the default
button cancels.

## Tag-driven release flow

Wixen Mail now has a dedicated release workflow:

- Workflow file: `.github/workflows/release.yml`
- GitHub Actions URL: `https://github.com/PratikP1/Wixen-Mail/actions/workflows/release.yml`
- Trigger: push to `main` (or run manually with `workflow_dispatch`)
- Manual run option: choose prerelease level input (`alpha`, `beta`, or `rc`; default `beta`)

## What the workflow produces

On Windows runners, the workflow runs a release quality gate (`cargo fmt --check`, clippy action, and `cargo test --quiet`), then uses `cargo release` to bump/tag and publishes:

- `Wixen-Mail-Setup-<tag>.exe`
- `Wixen-Mail-<tag>-windows.zip`
- `wixen-mail-<tag>.exe`

Release notes are generated automatically by GitHub between tags and attached to each GitHub Release along with `CHANGELOG.md`.

## Changelog and installer source

- [What changed in each version](changelog.md)
- Windows setup script: `installer/Wixen-Mail-Setup.iss` in the repository. Not linked,
  because these pages ship beside the program and the source does not.

## In-app update links

From **Help** menu:

- **Check for Updates** → latest release page
- **Release Pipeline** → release workflow run page

## What still needs to be done for the first beta release page

- Ensure branch protection on `main` requires the CI checks to pass before merge.
- Merge the release workflow changes to `main`.
- Trigger the release workflow once from `main` (push or `workflow_dispatch`) so `cargo release` creates the first beta tag and release.
- Verify the generated release has all expected assets (`setup`, zipped binary, standalone binary, and `changelog.md`) and mark it as **Pre-release**.
