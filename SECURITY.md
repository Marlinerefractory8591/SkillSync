# Security Policy

## Supported versions

Security fixes are applied to the current stable release on the `main` branch. The first stable release is `v1.0.0`.

## Reporting a vulnerability

Do not disclose vulnerabilities, access tokens, private keys, backup archives, customer data or private prompts in a public issue. Use the repository's private security advisory channel when it is enabled, or contact [Tomasz Bołoz](https://www.damtox.pl) with a concise reproduction, affected version and impact.

Please allow time for acknowledgement and remediation before publishing details. Reports involving arbitrary file writes, path traversal, unsafe Git operations, leaked credentials or update-integrity bypasses are especially valuable.

## Safe public diagnostics

Before sharing logs, redact GitHub tokens, authorization headers, local paths, repository URLs that are not public, prompt contents and backup filenames. SkillSync is designed to operate locally; users should still review each update target and changelog before confirming an update.
