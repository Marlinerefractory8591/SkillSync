# macOS Developer ID signing and notarization

Official SkillSync DMG releases are distributed outside the Mac App Store. To open without the misleading “application is damaged” Gatekeeper message, Apple requires two separate controls:

1. A **Developer ID Application** signature binds the app bundle to its identified developer.
2. **Notarization** has Apple scan the signed bundle and attach a ticket that Gatekeeper can validate offline.

The Tauri updater signature is a third, separate control. It verifies automatic updates inside SkillSync; it cannot replace an Apple Developer ID signature or notarization of a downloaded DMG.

## Required GitHub Actions secrets

Create these repository secrets in **Settings → Secrets and variables → Actions**. Never commit certificates, passwords or `.p8` files to Git.

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` export of the **Developer ID Application** certificate including its private key. |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting that `.p12` file. |
| `APPLE_API_KEY` | App Store Connect API key ID. |
| `APPLE_API_ISSUER` | App Store Connect issuer ID. |
| `APPLE_API_KEY_BASE64` | Base64-encoded contents of the App Store Connect API key `.p8` file. |
| `TAURI_SIGNING_PRIVATE_KEY` | Existing Tauri updater private key; separate from the Apple certificate. |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the Tauri updater private key. |

The workflow writes the API key to a temporary, permission-restricted runner file and exposes its path only for the Tauri build. It fails before packaging if any required Apple release secret is missing, so an unsigned DMG cannot be published by accident.

## Preparing secrets on macOS

Export the **Developer ID Application** certificate and private key from Keychain Access as a password-protected `.p12` file. Generate an App Store Connect API key with access to notarization, then keep its downloaded `.p8` file private.

Encode each file without adding a line break:

```bash
base64 -i DeveloperIDApplication.p12 | pbcopy
base64 -i AuthKey_ABC123DEFG.p8 | pbcopy
```

Paste the first result into `APPLE_CERTIFICATE` and the second into `APPLE_API_KEY_BASE64`. Put the key ID (`ABC123DEFG`) in `APPLE_API_KEY` and the issuer UUID in `APPLE_API_ISSUER`.

## What the release workflow verifies

For every pushed `v*.*.*` tag, the macOS job checks all of the following before upload:

```bash
codesign --verify --deep --strict --verbose=4 SkillSync.app
spctl --assess --type execute --verbose=4 SkillSync.app
xcrun stapler validate SkillSync.app
hdiutil verify SkillSync.dmg
xcrun stapler validate SkillSync.dmg
spctl --assess --type open --context context:primary-signature --verbose=4 SkillSync.dmg
```

The job also asserts that the signing authority starts with `Developer ID Application:`. A failed check stops the release, rather than creating an installer users cannot safely open.

## Verifying a published release locally

After downloading from the official GitHub release, compare its SHA-256 digest to `checksums.sha256`, mount it, then check the installed app:

```bash
shasum -a 256 SkillSync_*.dmg
hdiutil verify SkillSync_*.dmg
codesign --verify --deep --strict --verbose=4 /Applications/SkillSync.app
spctl --assess --type execute --verbose=4 /Applications/SkillSync.app
```

Do not bypass Gatekeeper for an official release. A Gatekeeper failure means the release must be investigated and republished through the signed workflow.
