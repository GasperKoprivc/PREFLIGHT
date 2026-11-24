# Build and release preparation

CI runs reusable quality checks on Windows/macOS/Linux: Rust fmt/check/Clippy/tests, SQLite API integration tests, and frontend lint/test/build. Successful quality checks are followed by unsigned installer builds: Windows x64 NSIS/MSI, native macOS runner architecture DMG, and Linux x64 deb/AppImage. Actions artifacts include SHA256SUMS and an unsigned-build notice, retained for 14 days.

These are **unsigned test builds**, not signed/notarized production releases. SmartScreen/Gatekeeper may block them. Follow the school's approved verification/signing process; do not disable security protections. Future signing needs separately managed Windows signing and Apple Developer ID/notarization credentials. None are configured here.

```sh
npm ci
npm run check:version
npm run bundle:desktop
```

The bundle command targets the local OS. On Windows, use `npm run tauri --workspace @preflight/desktop -- build --bundles nsis,msi` for both installers. NSIS installs per user; Git installation through winget can still require administrator confirmation. Linux artifacts use WebKitGTK 4.1 on Ubuntu 24.04 and are not claimed compatible with older distributions. AppImage needs a compatible runtime/FUSE environment; CI uses extract-and-run. macOS builds are architecture-specific, not universal.

## Smoke test on every target OS

Download the artifact for the exact commit and architecture and verify checksums. Launch it in an isolated test account. Test offline diagnostics, keyboard access, themes, invalid YAML, import/export, and individual rechecks. An unknown result must not imply readiness. With a test API/dashboard and two participants, verify opt-in, teacher refresh recovery, updates, unauthorized access, revocation and expiry. Git auto-install requires a deliberate user confirmation on Windows x64. Never perform installations in automated tests.

## Manual release workflow

The workflow_dispatch release-preparation workflow requires the main ref and explicit confirmation that manual smoke tests passed for that exact commit. It reruns quality and bundles. By default it uploads only Actions artifacts. The optional create_version_tag input creates v0.1.0 only after successful jobs and only if it does not exist; existing tags are never moved. A script first checks root npm, desktop npm, Tauri and Cargo workspace version alignment.

No public GitHub Release is created, no release assets are published, and ordinary pushes do not create a tag. Leave the tag option off until a user deliberately authorizes it after reviewing smoke-test evidence. A public release and production signing require separate explicit authorization. A green CI screenshot alone does not confirm manual smoke tests.

Record pending verification honestly: remote artifacts and three-OS smoke tests remain unverified until reviewed. See docs/pilot.md for the acceptance record.

`npm run smoke:api` builds and launches an isolated API with ephemeral in-memory SQLite, exercises two participants, opt-in upload, unauthorized access, revocation and deletion, then stops that process. No real room credentials are printed or persisted. The quality workflow runs it on every OS. See docs/security-review.md for the current advisory findings, including unresolved Linux/GTK informational warnings; review these before any release decision.

Official references: https://v2.tauri.app/distribute/windows-installer/ , https://v2.tauri.app/distribute/appimage/ , https://v2.tauri.app/distribute/sign/macos/ .
