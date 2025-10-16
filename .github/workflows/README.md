# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the FGBrowser desktop application.

## Workflows

### build.yml

**Trigger:** Push to main, Pull requests to main, Manual dispatch

Builds the Tauri desktop application for all three platforms:

- Windows (MSI and NSIS installers)
- macOS (Universal binary for Intel and Apple Silicon)
- Linux (DEB and AppImage)

The built artifacts are uploaded and can be downloaded from the Actions tab.

### build-cli.yml

**Trigger:** Push to main, Manual dispatch

Builds the CLI binary for Linux (x86_64). The binary is uploaded as an artifact for testing purposes. For official releases, use the release workflow below.

### release.yml

**Trigger:** Git tags starting with `v*`, Manual dispatch

Creates a unified GitHub release with all distribution packages. To create a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This will build and release:

1. **Desktop Apps:**
   - Windows: MSI and NSIS installers
   - macOS: Universal DMG (Intel + Apple Silicon)
   - Linux: DEB and AppImage packages

2. **CLI Binary:**
   - Linux: Standalone x86_64 binary

All artifacts are uploaded to a single draft release, allowing you to review and edit before publishing.

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
