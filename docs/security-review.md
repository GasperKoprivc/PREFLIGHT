# Release-readiness security review

Reviewed locally on 2026-11-24. This is a scoped engineering review, not a guarantee of security or GDPR compliance.

## Controls inspected

- Tauri loads a bundled frontend under a restrictive CSP; capabilities expose core defaults for the main window, with no shell/filesystem plugin. Native operations use explicit Rust commands.
- Profiles use typed allowlists and size/depth/count limits. They cannot supply executable names or arguments. Runtime checks and the installer run directly, without a shell.
- FixAction is limited to Windows x64 Git.Git 2.56.1, with fixed source/version/architecture. Preparation validates the local profile/result; execution consumes an expiring one-use confirmation and locks concurrent operations. No real installation occurs in tests.
- HTTPS diagnostics validate every target and redirect, reject private/loopback/reserved addresses, and pin vetted DNS addresses. Classroom clients accept an explicitly entered HTTPS origin or development loopback, with redirects disabled and bounded responses.
- Teacher and participant bearer tokens are independent, stored hashed in SQLite and compared in constant time. Uploaded results are allowlisted and minimized; opt-in, cross-session authorization, expiry and cascade deletion have integration tests. SQLite SQL uses bound parameters.
- Sessions and result payloads do not include OS usernames, machine IDs, raw command logs or paths. User-authored profile IDs/names still must not contain personal information. Proxy logs and backups require operator controls.

## Dependency findings and disposition

The first cargo-audit run found libsqlite3-sys 0.30.1 (RUSTSEC-2026-0459) and rsa 0.9.10 (RUSTSEC-2023-0071) in the lockfile. SQLx was upgraded to 0.9.1 and its compatible SQLite dependency to 0.39.1. The RSA dependency was removed from the lockfile by that upgrade. API migrations and authorization tests were rerun successfully. SQLx now requires Rust 1.94+.

The subsequent audit returned zero entries in its vulnerabilities list, but retained these **informational warnings**, which must not be described as a clean security bill:

- RUSTSEC-2024-0429: glib 0.18.5 VariantStrIter unsoundness, fixed upstream in >=0.20. Tauri's current GTK3/Linux dependency chain requires the 0.18 family; replacing it with an incompatible glib major is not a safe lockfile-only fix. Application code does not directly use VariantStrIter, but transitive use has not been proven unreachable. Linux release readiness remains qualified pending upstream resolution or explicit technical risk assessment and target testing.
- RUSTSEC-2024-0370: proc-macro-error is unmaintained in the Linux GTK build chain. No compatible fixed release is reported.
- RUSTSEC-2026-0543: chrono 0.4.45 is unmaintained in the resolved lockfile; cargo tree with all targets found no active path in the current feature selection. It is not used directly by the application.

No advisory is suppressed. CI runs cargo-audit with its default policy: vulnerabilities fail the job, informational warnings remain visible. npm audit reported zero vulnerabilities. Re-evaluate the advisory database before tagging or production deployment; do not treat informational warning status as a proof of harmlessness.

## Open acceptance evidence

Actual Git installation/UAC confirmation remains a user-driven smoke test. Windows installer bundles were built locally; cross-OS package installation and manual target smoke tests require user review. No production service, signed/notarized build, version tag or public release was created. See docs/release.md and docs/pilot.md for the remaining evidence.
