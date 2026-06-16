# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
