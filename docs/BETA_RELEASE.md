# Beta Release and Setup Executables

## Tag-driven release flow

Wixen Mail now has a dedicated release workflow:

- Workflow file: `.github/workflows/release.yml`
- GitHub Actions URL: `https://github.com/PratikP1/Wixen-Mail/actions/workflows/release.yml`
- Trigger: push a tag like `v0.1.0-beta.1` (or run manually with `workflow_dispatch`)

## What the workflow produces

On Windows runners, the workflow builds release artifacts and publishes:

- `Wixen-Mail-Setup-<tag>.exe`
- `Wixen-Mail-<tag>-windows.zip`

For tag pushes, assets are attached to a GitHub Release automatically.

## In-app update links

From **Help** menu:

- **Check for Updates** → latest release page
- **Release Pipeline** → release workflow run page
