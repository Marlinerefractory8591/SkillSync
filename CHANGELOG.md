# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-09-16

### Added

- Desktop management of valid AI-agent skills for Claude Code, Codex, Cursor, Gemini and custom monitored paths.
- Safe snapshots and rollback before skill updates.
- Native release builds for macOS Apple Silicon, macOS Intel and Windows, with SHA-256 checksums.
- Polish and English README guides, screenshots, direct-answer FAQs and release documentation.

### Fixed

- Reject ordinary `package.json` directories such as `docs`, `gallery` and workspace packages during skill discovery and update validation.
- Offer a confirmation before a forced update of a Git worktree containing modified tracked files.
- Treat a missing local version as unknown rather than `v1.0.0`.
- Apply SemVer prerelease precedence, so stable `1.9.6` does not report `1.9.6-codex.5` as an available update.

### Security

- Do not overwrite unmarked package manifests during updates.
- Validate the skill manifest after every update and restore the snapshot when validation fails.

[1.0.0]: https://github.com/tomaszboloz/SkillSync/releases/tag/v1.0.0
