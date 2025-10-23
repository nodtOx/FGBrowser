# Changelog

All notable changes to this project will be documented in this file.

**Note:** This changelog is manually curated. Before each release, review your commits and add user-facing descriptions here. Use `git log v0.1.18..HEAD --oneline` to see what changed since the last release.

## [Unreleased]

### Added

### Changed

### Fixed

## [0.2.0] - 2025-10-23

### Added

- View high-resolution screenshots and videos for each game
- Full-screen screenshot viewer with smooth animations and zone-based navigation
- Click left/right sides of screenshots to browse, center to close
- Thumbnail support for faster screenshot loading
- Preloading of full-resolution images for instant lightbox viewing
- 4-directional keyboard navigation in grid view (all arrow keys)
- Keyboard shortcut 'V' to toggle between list and grid views
- Visual focus indicators for grid and list panels

### Changed

- Improved keyboard navigation: arrow keys now control screenshots in game details
- Backspace (not Escape) now returns to game list to avoid lightbox conflicts
- Game details page only renders when opened (performance improvement)
- Status bar shortcuts updated to reflect new keyboard controls

### Fixed

- Grid view keyboard navigation now properly handles 2D movement
- Tab key panel switching now works correctly in grid mode
- Screenshot grid no longer jumps during window resize

## [0.1.18] - 2024-10-19

### Changed

- Updated README to reflect active development status
- Clarified telemetry usage and added roadmap section to documentation

## [0.1.17] - 2024-10-18

### Added

- NEW games feature with is_seen tracking
- Visual NEW badges for recently added games

### Fixed

- Disabled Sentry in development mode
- Corrected NEW badge positioning
- Resolved Tokio runtime panic on macOS

## [0.1.16] - 2024-10-17

### Added

- Auto game updater functionality (PR #2)
- Automatic update checking and installation

## [0.1.15] - 2024-10-17

### Fixed

- Corrected updater permissions for Tauri v2

## [0.1.14] - 2024-10-17

### Changed

- Removed deprecated GitHub workflows for CLI and release processes

### Fixed

- Added updater ACL permissions for Tauri v2

## [0.1.13] - 2024-10-16

### Changed

- Updated package-lock.json version to 0.1.11

### Documentation

- Added Windows continuation guide for auto-updater testing

## [0.1.12] - 2024-10-16

### Changed

- Version bump for updater testing

## [0.1.11] - 2024-10-16

### Added

- Windows-only release (removed macOS builds)

### Changed

- Updated Homebrew SHA256 for v0.1.10

## [0.1.10] - 2024-10-15

### Changed

- Removed Apple Developer certificate steps

## [0.1.9] - 2024-10-15

### Fixed

- Used correct Tauri environment variable names for signing

### Performance

- Added Rust/Cargo caching to GitHub Actions

## [0.1.8] - 2024-10-14

### Added

- Regenerated updater keys with password for better security

### Fixed

- Watch Release workflow instead of CLI build

### Changed

- Updated Homebrew SHA256 for v0.1.7

## [0.1.7] - 2024-10-14

### Added

- Auto-commit in bump script for streamlined workflow
- Configured Tauri auto-updater with signing keys

### Fixed

- Corrected Intel DMG filename pattern from x86_64 to x64
- Properly get workflow run ID for gh run watch

### Changed

- Updated Homebrew SHA256 for v0.1.6

## [0.1.6] - 2024-10-13

### Added

- Husky pre-commit hooks
- Entitlements validation command
- All-in-one release with live GitHub Actions logs

### Fixed

- Removed deprecated Husky v10 lines from pre-commit hook
- Corrected entitlements.plist XML syntax and prevent auto-formatting

## [0.1.5] - 2024-10-13

### Added

- Support for separate GitHub account via token
- Automated Homebrew SHA256 update after GitHub Actions completes
- Homebrew Cask distribution support

### Changed

- Calculate Homebrew SHA256 locally instead of in GitHub Actions
- Updated bump script to include Homebrew cask version

### Documentation

- Added macOS installation instructions to release notes

## [0.1.4] - 2024-10-12

### Fixed

- Added macOS entitlements and ad-hoc signing to fix launch issues

## [0.1.3] - 2024-10-12

### Fixed

- Updated tag push command in release script to specify origin

## [0.1.2] - 2024-10-12

### Added

- Enhanced auto-update functionality
- Application screenshot in README

### Changed

- Removed Apple signing identity from workflow to skip code signing

## [0.1.1] - 2024-10-11

### Added

- Versioning system
- Separated CLI from desktop app

## [0.1.0] - 2024-10-11

### Added

- Initial release
- Database download functionality
- Popular repacks fetching and crawling
- Pink Paw Award feature
- Image caching
- View mode toggle (list/grid)
- Keyboard navigation
- Game filtering by category, size, and time
- Game search functionality
- Download management features
- Theme management with dark/light mode
- GitHub Actions CI/CD
- Multi-site crawler architecture
- Rust-based crawler (migrated from Python)
- Category management and normalization
- Unseen popular games tracking

### Fixed

- GitHub Actions v4 artifacts support
- GitHub releases permissions

### Performance

- Database optimizations
- Polling interval improvements

[unreleased]: https://github.com/nodtOx/FGBrowser/compare/v0.1.18...HEAD
[0.1.18]: https://github.com/nodtOx/FGBrowser/compare/v0.1.17...v0.1.18
[0.1.17]: https://github.com/nodtOx/FGBrowser/compare/v0.1.16...v0.1.17
[0.1.16]: https://github.com/nodtOx/FGBrowser/compare/v0.1.15...v0.1.16
[0.1.15]: https://github.com/nodtOx/FGBrowser/compare/v0.1.14...v0.1.15
[0.1.14]: https://github.com/nodtOx/FGBrowser/compare/v0.1.13...v0.1.14
[0.1.13]: https://github.com/nodtOx/FGBrowser/compare/v0.1.12...v0.1.13
[0.1.12]: https://github.com/nodtOx/FGBrowser/compare/v0.1.11...v0.1.12
[0.1.11]: https://github.com/nodtOx/FGBrowser/compare/v0.1.10...v0.1.11
[0.1.10]: https://github.com/nodtOx/FGBrowser/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/nodtOx/FGBrowser/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/nodtOx/FGBrowser/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/nodtOx/FGBrowser/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/nodtOx/FGBrowser/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/nodtOx/FGBrowser/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/nodtOx/FGBrowser/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/nodtOx/FGBrowser/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/nodtOx/FGBrowser/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/nodtOx/FGBrowser/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nodtOx/FGBrowser/releases/tag/v0.1.0
