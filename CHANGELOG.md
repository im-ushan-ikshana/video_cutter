# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.4] - 2026-06-22

### Added
- Added global, full-window drag-and-drop support natively through the Tauri API, complete with a beautiful, theme-aware drop overlay.
- Added previously unmentioned keyboard shortcuts (Fast Forward via `Hold →`, Rewind via `Hold ←`) to the Settings dialog.

### Changed
- Replaced the native OS dropdown in Export Settings with a fully custom, theme-aware interactive dropdown menu.
- Redesigned the main Video Preview empty state to look sleeker and include a drag-and-drop hint.
- Increased the width of the Settings Dialog to comfortably accommodate the expanded keyboard shortcuts list.

## [1.0.3] - 2026-06-17

### Security
- Dramatically hardened the Tauri application webview by enforcing a strict Content Security Policy (CSP) that mathematically guarantees the application cannot instantiate unauthorized external internet connections or execute remote scripts.
- Configured the Tauri Asset Protocol to securely serve media payloads across Windows (`http://asset.localhost`) and UNIX (`https://asset.localhost`) boundaries without violating the CSP.
- Maintained global filesystem read access (`**`) via the Asset Protocol exclusively to ensure seamless editing of massive video files stored on external USB drives, NAS networks, and secondary partitions.

### Fixed
- Fixed an issue where the Video.js player icons (play, pause, volume) failed to render in the production build due to the strict Content Security Policy blocking Vite's compiled font `data:` URIs and Tauri's custom protocols.

## [1.0.2] - 2026-06-16

### Added
- Introduced a highly robust, interactive PowerShell script (`local_release.ps1`) for compiling and pushing MSI/NSIS bundles natively to GitHub Releases.
- Added intelligent FFmpeg caching to skip redundant sidecar downloads during local builds.
- Placed a beautiful SVG application banner into the repository `README.md`.

### Fixed
- Repaired the unresponsive GitHub profile link in the Settings dialog by migrating it to Tauri's official `plugin-opener`.
- Hardened the local release pipeline to aggressively purge old compilation targets and enforce strict regex artifact filtering to prevent accidental cross-contamination.

## [1.0.1] - 2026-06-16

### Fixed
- Disabled the default browser right-click context menu globally across the application.
- Enforced single-instance mode; launching the application multiple times will now gracefully pull the existing active window to the foreground instead of spawning clones.

## [1.0.0] - 2026-06-16

### Added
- First official release of Universal Video Cutter!
- Native multi-codec video export (H.264, HEVC, VP9).
- Instant "Original (Fast Cut)" mode for zero-re-encoding trims.
- Hardware-accelerated proxy generation mechanism for smooth scrubbing of non-native formats.
- Complete Windows desktop integration with custom draggable titlebar and dark/light mode detection.
- Intelligent timeline scrubbing with infinite mouse sliding.
- Automatic temporary file cleanup routine running every 6 hours to prevent drive bloat.
- Beautiful, fully responsive UI built with SvelteKit and Tailwind CSS.
- Secure, sandboxed Tauri Rust backend to manage heavy FFmpeg processing safely.
