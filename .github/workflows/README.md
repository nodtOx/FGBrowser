# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the Fit Boy desktop application.

## Workflows

### build.yml

**Trigger:** Push to main, Pull requests to main, Manual dispatch

Builds the Tauri desktop application for all three platforms:

- Windows (MSI and NSIS installers)
- macOS (Universal binary for Intel and Apple Silicon)
- Linux (DEB and AppImage)

The built artifacts are uploaded and can be downloaded from the Actions tab.

### release.yml

**Trigger:** Git tags starting with `v*`, Manual dispatch

Creates a GitHub release with installers for all platforms. To create a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This will:

1. Build the app for all platforms
2. Create a draft release on GitHub
3. Upload all installers to the release

The release will be created as a draft, allowing you to review and edit it before publishing.

### check.yml

**Trigger:** Push to main, Pull requests to main

Runs quality checks on the codebase:

- Frontend: Svelte type checking
- Backend: Rust tests, formatting check, and Clippy lints

This ensures code quality before merging changes.

## Requirements

No special setup is required. The workflows will:

- Install Node.js and Rust automatically
- Cache dependencies for faster builds
- Handle platform-specific requirements

## Notes

- macOS builds create universal binaries that work on both Intel and Apple Silicon Macs
- Linux builds require GTK and WebKit dependencies (installed automatically in CI)
- Windows builds create both MSI and NSIS installers
- Builds only trigger when files in the `desktop/` directory change
