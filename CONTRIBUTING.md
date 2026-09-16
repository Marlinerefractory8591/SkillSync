# Contributing to SkillSync

Thank you for improving SkillSync. Please keep each contribution small, testable and safe for public distribution.

## Before opening a pull request

1. Create a branch from `main` and describe the user-visible outcome in the commit message using Conventional Commits.
2. Run `npm run test:unit`, `npm run lint`, `npm run build`, and `cargo test --manifest-path src-tauri/Cargo.toml`.
3. Do not add `node_modules`, `dist`, `dist-release`, `src-tauri/target`, backups, local settings, `.env` files, screenshots with personal paths, tokens, private prompts or customer data.
4. Explain any change to skill discovery, updating, rollback or manifest validation and add a regression test.

## Release policy

Create annotated stable tags as `vMAJOR.MINOR.PATCH`; use `-rc.N`, `-beta.N`, or `-codex.N` for prereleases. See [docs/SEMVER_RELEASE.md](docs/SEMVER_RELEASE.md) for precedence and publication details.

## Reporting ordinary bugs

Use a GitHub issue with the app version, operating system, reproducible steps, expected result and actual result. Replace home-directory paths with placeholders such as `/Users/you/...` before posting.
