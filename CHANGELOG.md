# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [Semantic Versioning](https://semver.org/).

## [1.2.0] - 2026-09-18

### Added

- Track the detected Git branch for Git-backed Skills, MCP integrations and plugins, with a per-package manual branch override that always takes precedence.
- Add the **Bez gałęzi** filter to identify Git packages that require a tracking branch before they can be monitored against upstream commits.
- Add a persistent system menu-bar / system-tray icon setting. The icon restores the window on click and exposes explicit Show and Quit actions.

### Changed

- Split local package discovery from upstream verification. The scan returns local results immediately, then a single background worker checks one upstream source at a time with a 350 ms spacing and streams each result back to its card.
- Compare tracked branch commits with `origin` without modifying the local worktree. A branch or commit mismatch is presented as an actionable update; updating safely fetches and checks out the selected branch.
- Display non-SemVer references such as `main` without a misleading `v` prefix.

## [1.1.2] - 2026-09-17

### Fixed

- Make bulk discovery use a bounded GitHub verification queue instead of opening an unbounded burst of upstream requests.
- Retry a transiently unavailable GitHub source once and give the card an actionable error state rather than incorrectly reporting that it is up to date.
- Use one release-application path for scanning and the per-item **Check GitHub** action, so both surface the same current upstream version and update decision.

## [1.1.1] - 2026-09-17

### Fixed

- Check for a new SkillSync release automatically on application launch when the setting is enabled.
- Show the installed and available application version in a persistent bottom footer, with a manual refresh action.
- Download, verify with the embedded public key, install and restart signed SkillSync updates directly from the persistent footer.
- Reserve bottom content space so the fixed footer cannot cover the final cards.

### Security

- Publish a signed Tauri updater package and `latest.json` manifest for macOS and Windows; the private signing key is kept only in GitHub Actions secrets.

## [1.1.0] - 2026-09-16

### Added

- Unified discovery, filtering and sorting for Skills, MCP integrations and agent plugins.
- Dedicated Composer update adapter for installed Laravel Boost, including Composer validation, project tests and lockfile verification.
- Registry-aware Claude Code Marketplace plugin updates through the official `claude plugin update` command.

### Fixed

- Ignore obsolete Claude Code cache copies and use the active `installed_plugins.json` entry, preventing stale plugins such as `n8n-mcp-skills` from failing as non-Git directories.
- Make rollback exact by removing files created during a failed update before restoring the snapshot.
- Do not display a hard-coded application version in the navigation header.

### Security

- Keep plugin registry ownership with Claude Code rather than mutating Marketplace cache directories directly.
- Require an explicit supported manifest and preflight before every managed update.

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
[1.2.0]: https://github.com/tomaszboloz/SkillSync/releases/tag/v1.2.0
[1.1.0]: https://github.com/tomaszboloz/SkillSync/releases/tag/v1.1.0
[1.1.1]: https://github.com/tomaszboloz/SkillSync/releases/tag/v1.1.1
[1.1.2]: https://github.com/tomaszboloz/SkillSync/releases/tag/v1.1.2
