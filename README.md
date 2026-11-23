# Preflight

Preflight checks your local development environment before a workshop. The desktop application and CLI share the same Rust core. Results stay on your computer; teacher rooms and result sharing are planned for later phases.

## Development

Requirements: Node.js 24+, npm, stable Rust, and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/). Windows requires MSVC C++ Build Tools and WebView2; macOS requires Xcode Command Line Tools; Linux requires WebKitGTK 4.1 and the libraries listed in the CI workflow. Both cargo and npm must be on PATH. In PowerShell, use `npm.cmd` if script execution restrictions prevent running npm.

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

Profiles use `schema_version: 1`, unique IDs, and only `type: runtime` with `tool: python|git|node|java`. Java means JDK: both java and javac are checked. Version constraints follow Rust semver, for example `>=22.0.0,<23.0.0`; a bare `22.0.0` specifies a compatible range, while `=22.0.0` requires an exact version. Legacy Java versions such as 1.8.0_412 are normalized to 8.0.412. Unstable or unrecognized output produces an unknown result. YAML is limited to 64 KiB, 32 checks, and a nesting depth of 8; unknown fields, tags, aliases, and multiple documents are rejected.

Each process has a 5-second timeout and a maximum combined output of 32 KiB; stdout and stderr are read concurrently with bounded buffers. Processes are spawned directly with static arguments, without a shell, and stopped when a limit is reached. Python lookup: Windows tries python, then py -3; Unix tries python3, then python. JSON contains no raw output, profile paths, or machine information. Check IDs come from the profile: do not put personal information in them. Do not commit diagnostic output to Git.

## Desktop overview (phase 03)

`npm run dev:desktop` opens the desktop application. The interface is currently in Slovenian. Select the Python workshop, click “Zaženi preverjanje” (Run checks), expand a row for an explanation, and click “Ponovno preveri” (Check again). A required unknown result prevents the environment from being marked ready. Profile selection and repeated execution are locked while checks run, including in the Rust layer.

The Privacy and Settings views explain local processing and offer system, light, and dark themes. Keyboard navigation uses Tab to move and Enter/Space to activate buttons and details. The interface includes visible focus indicators, a skip link, and a live results summary. Results remain in memory; only the theme preference is saved. The application calls preflight-core directly, without a CLI subprocess or API server requests.

This phase includes one built-in profile. Profile import, additional diagnostics, and export are planned for phase 04. Teacher sharing is not available yet.
