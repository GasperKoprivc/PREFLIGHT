# API deployment

For a same-origin HTTPS example, see `deploy/Caddyfile.example`. Replace the hostname and dashboard directory, configure DNS/firewall/TLS, and use the operator's approved deployment process. The example is not a deployed service. Add proxy abuse limits, connection limits and monitoring appropriate to the school. Keep SQLite on a persistent disk outside the web root, restricted to the service account.

Build the dashboard with `npm run build --workspace @preflight/dashboard` and serve its dist folder. Forward `/api/*` and `/health` to the loopback API. Prefer same-origin hosting with CORS unset; use one exact allowed origin only for an intentional cross-origin setup. Never record Authorization, payloads or identifying query strings. Monitor cleanup failures and disk capacity. Test backup/restore in isolation and enforce short backup retention.

See docs/release.md for unsigned installer limits and docs/pilot.md for the first 10–15 participant pilot.

Run `cargo run -p preflight-server` from the repository root. SQLite is created on first startup and migrations in `apps/server/migrations` run automatically. No external database or SQLx CLI is needed. Configuration is read from process environment; `.env.example` is a reference, not auto-loaded.

| Variable | Default | Meaning |
| --- | --- | --- |
| DATABASE_URL | sqlite://preflight.sqlite | SQLite database on a persistent writable disk |
| PREFLIGHT_BIND | 127.0.0.1:3001 | Listening address |
| PREFLIGHT_ROOM_TTL_SECONDS | 86400 | Room lifetime, 60–86400 seconds |
| PREFLIGHT_ALLOWED_ORIGIN | unset | One exact origin, e.g. https://preflight.example, without trailing slash |

PowerShell example:

```powershell
$env:DATABASE_URL = 'sqlite://preflight.sqlite'
$env:PREFLIGHT_BIND = '127.0.0.1:3001'
cargo run -p preflight-server
```

Host the dashboard and API on the same HTTPS origin where possible. CORS is closed by default; configure an exact origin only when required, never `*`. CORS is not authentication. Use a TLS reverse proxy, forward only to the loopback listener, limit request size/time/concurrency and apply additional abuse limits, especially to room creation and joining. Do not put bearer tokens in URLs, logs or analytics. Use a dedicated unprivileged service account and restrict SQLite file permissions. This phase does not publish a server or provide production certificates.

The built-in limiter is a process-wide 600 requests/minute fixed window with no IP tracking. Capacity is limited to 1,000 rooms and 200 sessions/room. Run one server process for this MVP; SQLite and process-local limits are not designed for multi-instance scaling. Monitor health, storage, cleanup errors and generic failure counts without storing request bodies or identifiers. The cleanup task logs only a static error message if deletion fails; investigate storage availability and restart after fixing it.

Use persistent storage and back up through SQLite's online backup API or stop the service before copying the database. Do not copy a live database file in isolation, particularly if journal/WAL mode is changed later. Backups contain profiles and shared results: encrypt them, restrict access, and set a short retention period consistent with room expiry. On restoration, startup cleanup removes expired rooms. Never commit databases or real diagnostic results to Git.

Local validation uses in-memory SQLite with real migrations and HTTP router tests. The API does not fetch profile URLs or run profile checks: clients receive a validated profile and execute checks locally. Dashboard/desktop classroom integration is phase 06.
