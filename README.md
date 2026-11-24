# Preflight

## Test installers and pilot readiness (phase 09)

After three-OS quality checks, CI builds unsigned Windows NSIS/MSI, native-architecture macOS DMG and Linux deb/AppImage artifacts, including checksums and a test-build notice. Installer creation does not certify a completed installation or platform smoke test.

Use `npm run bundle:desktop` for a local bundle and `npm run check:version` for version alignment. The manual release-preparation workflow requires reviewed main and explicit smoke-test confirmation. Its optional v0.1.0 tag follows successful checks/bundles; it never automatically publishes a public GitHub Release. No tag/release is created while implementing this phase.

See [release checks](docs/release.md), [deployment](docs/deployment.md), [privacy](docs/privacy.md) and the [10–15 participant pilot guide](docs/pilot.md).

Preflight checks your local development environment before a workshop. The desktop application and CLI share the same Rust core. Results stay on your computer; teacher rooms and result sharing are planned for later phases.

## Development

Requirements: Node.js 24+, npm, stable Rust 1.94+ (SQLx requirement), and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/). Windows requires MSVC C++ Build Tools and WebView2; macOS requires Xcode Command Line Tools; Linux requires WebKitGTK 4.1 and the libraries listed in the CI workflow. Both cargo and npm must be on PATH. In PowerShell, use `npm.cmd` if script execution restrictions prevent running npm.

From the repository root:

```sh
npm ci
npm run dev:desktop
npm run dev:dashboard
npm run dev:server
cargo run -p preflight-cli -- --help
```

Run each development command in a separate terminal. The dashboard runs at http://127.0.0.1:1421 and the API at http://127.0.0.1:3001/health (returns `ok`). The desktop Vite server uses port 1420; Rust commands work only inside the Tauri window.

```sh
npm run build
npm run lint
npm test
npm run build:desktop
```

`build` builds both frontends and the entire Cargo workspace. `build:desktop` creates a production desktop executable without an installer. `lint` checks ESLint, TypeScript, Rust formatting, and Clippy. CI checks all three operating systems. Installers and code signing are planned for later phases.

## Modules and roadmap

- `crates/preflight-core`: shared diagnostic core.
- `crates/preflight-cli`: the `preflight` command.
- `apps/desktop`: React/TypeScript/Vite + Tauri 2, with an explicit `get_app_info` Rust command.
- `apps/server`: Axum API, currently exposing only `/health`, bound to loopback.
- `apps/dashboard`: a clearly labeled “in preparation” placeholder.

Phase 00 sets up the repository; phase 01 adds the project scaffold; phase 02 adds typed profiles and CLI checks; phase 03 adds the offline desktop results interface. See [ROADMAP.md](ROADMAP.md) for the full plan, [AGENTS.md](AGENTS.md) for project conventions, and [START_HERE.md](START_HERE.md) for initial setup.

The implementation uses [official Tauri IPC](https://v2.tauri.app/develop/calling-rust/) and [Vite](https://vite.dev/guide/). JavaScript has no general shell or filesystem permissions. Both lockfiles are committed to Git.

## CLI checks

```sh
cargo run -p preflight-cli -- check --profile profiles/python-workshop.yaml --format text
cargo run -p preflight-cli -- check --profile profiles/python-workshop.yaml --format json
```

Exit codes: 0 = all required checks pass; 1 = at least one required check does not pass (including unknown results); 2 = an invalid or unreadable profile, or incorrect CLI usage. An optional version mismatch produces a warning. `unknown` never counts as a successful check.

Profiles use `schema_version: 1`, unique IDs, and the allowed types `runtime`, `vscode_extension`, `disk_space`, and `https`. Runtime checks use `tool: python|git|node|java`. Java means JDK: both java and javac are checked. Version constraints follow Rust semver, for example `>=22.0.0,<23.0.0`; a bare `22.0.0` specifies a compatible range, while `=22.0.0` requires an exact version. Legacy Java versions such as 1.8.0_412 are normalized to 8.0.412. Unstable or unrecognized output produces an unknown result. YAML is limited to 64 KiB, 32 checks, and a nesting depth of 8; unknown fields, tags, aliases, and multiple documents are rejected.

Each process has a 5-second timeout and a maximum combined output of 32 KiB; stdout and stderr are read concurrently with bounded buffers. Processes are spawned directly with static arguments, without a shell, and stopped when a limit is reached. Python lookup: Windows tries python, then py -3; Unix tries python3, then python. JSON contains no raw output, profile paths, or machine information. Check IDs come from the profile: do not put personal information in them. Do not commit diagnostic output to Git.

## Desktop overview (phase 03)

`npm run dev:desktop` opens the desktop application. The interface is currently in Slovenian. Select the Python workshop, click “Zaženi preverjanje” (Run checks), expand a row for an explanation, and click “Ponovno preveri” (Check again). A required unknown result prevents the environment from being marked ready. Profile selection and repeated execution are locked while checks run, including in the Rust layer.

The Privacy and Settings views explain local processing and offer system, light, and dark themes. Keyboard navigation uses Tab to move and Enter/Space to activate buttons and details. The interface includes visible focus indicators, a skip link, and a live results summary. Results remain in memory; only the theme preference is saved. The application calls preflight-core directly, without a CLI subprocess or API server requests.

Phase 03 introduced one built-in profile. Phase 04 extends this with the profiles and import/export features described below. Teacher sharing in the desktop UI is not available yet.

## Profiles and extended checks (phase 04)

Three examples are included in `profiles/`: Python, Java/JDK, and web development. The desktop **Profili** view opens a local YAML file, edits it, validates it through Rust, applies it for the current session, and exports it through a native save dialog. Export edited profiles before closing the app. **Izvozi JSON / Izvozi besedilo** saves the latest results locally.

Additional allowed check types:

```yaml
- id: vscode-python
  type: vscode_extension
  extension: ms-python.python
  required: true
- id: disk
  type: disk_space
  min_free_gb: 5
  required: true
- id: pypi
  type: https
  url: https://pypi.org
  required: false
```

Disk space is measured on the filesystem containing the current working directory; GB means 1,000,000,000 bytes. VS Code checks the default extension profile, using the CLI or a known installation fallback. An unavailable CLI returns unknown; other editors, portable installations and custom VS Code profiles may need manual verification.

Desktop HTTPS checks are off by default. Enabling them contacts the listed servers, which can see your IP address, but does not send diagnostic results. CLI checks HTTPS entries unless `--offline` is supplied. HTTPS checks use HEAD without downloading response bodies, verified TLS, port 443, a 12-second overall timeout and at most three redirects. Every hop resolves and validates all addresses, then pins those addresses for the request; private, loopback, link-local, metadata and reserved ranges are blocked. Proxies are disabled. A server that rejects HEAD produces a warning rather than a false pass.

```sh
cargo run -p preflight-cli -- check --profile profiles/python-workshop.yaml --format text --offline
```

Only profile-defined IDs and normalized versions appear in exported results; do not put personal data in profile IDs or names. Profiles cannot specify executable commands or script arguments.

## Classroom API (phase 05)

`npm run dev:server` starts the Axum/SQLite API at `127.0.0.1:3001`. SQLite migrations run automatically; configure it with the environment variables in `.env.example`. Teacher rooms expire after 24 hours by default. Teacher and participant bearer tokens have separate permissions; result uploads require explicit consent and accept only minimized structured data.

See [API endpoints and payloads](docs/api.md), [privacy and retention](docs/privacy.md), and [deployment configuration](docs/deployment.md). The teacher dashboard and desktop room-sharing flow are planned for phase 06; phase 05 is a working API.

```sh
cargo test -p preflight-server
cargo run -p preflight-server
```

## Teacher dashboard and voluntary sharing (phase 06)

Run the API, dashboard, and desktop in separate terminals:

```sh
npm run dev:server
npm run dev:dashboard
npm run dev:desktop
```

Open http://127.0.0.1:1421, paste a valid YAML profile and create a room. The development dashboard proxies `/api` to the API on port 3001; production must serve `/api` from the same origin. The teacher token is masked and stored only in `sessionStorage` for the current tab so refresh preserves control. Save it securely before closing the tab; the recovery form accepts it without putting it in a URL. Shared invite links contain only the public room code.

In the desktop **Soba** view, enter the API origin (development: `http://127.0.0.1:3001`) and room code. Download/review the profile, run local checks in **Pregled okolja**, return to **Soba**, inspect the minimized JSON, explicitly consent and click **Deli rezultate**. Each update requires a new consent and click. The Rust layer checks that the latest report belongs to the unchanged room profile. Tokens remain in Rust memory; the frontend does not receive them. **Izklopi deljenje in izbriši sejo** deletes the server session; on a network failure it keeps the token for retry. Delete before closing the app, otherwise the shared session remains until expiry and the in-memory token is lost.

The dashboard polls every five seconds, shows the last successful refresh, problematic-check filters, empty/unshared states and expiry/authentication errors. It stops polling on expiry/unauthorized access. Participant reports are not trusted attestation. The desktop accepts HTTPS API origins, plus HTTP loopback for development, with no redirects and bounded responses. Local diagnostics remain independent of classroom services.

## Guided fixes (phase 07)

Expand a desktop check result to see Slovenian guidance for its reason and operating system. The OS defaults to the native Rust platform and can be changed manually. The catalog covers missing runtimes, version/PATH mismatches, JDK problems, VS Code CLI/extensions, disk, HTTPS and unknown/timeout results. Official documentation and static copyable verification/install commands are included; no installation is executed by Preflight. Unknown extension IDs never become commands.

**Ponovno preveri to zahtevo** rechecks only that ID from the validated profile and recomputes the summary, preserving other results from their previous check. A full report for the unchanged profile is required first. Rechecking does not share results automatically. See [troubleshooting](docs/troubleshooting.md) for platform details and limitations.

## Confirmed installation (phase 08)

The initial automatic-fix catalog supports only Git for Windows x64 via the fixed `Git.Git` 2.56.1 winget package, when that version matches the profile. Open the Git result, review the full action and explicitly click **Potrdi namestitev**. The backend uses an expiring single-use confirmation, a fixed argument registry, bounded execution and a global operation lock. Other platforms/tools retain manual guidance.

See [supported actions, confirmation flow and smoke-test limits](docs/safe-installs.md). Package metadata and winget were checked on Windows; actual installation through the UI has not been physically confirmed. Tests use fake runners and never install software. The phase 08 manual-install acceptance check remains pending that explicit user smoke test.
