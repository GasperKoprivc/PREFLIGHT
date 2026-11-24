# Classroom API (phase 05)

Start with `cargo run -p preflight-server`. Migrations run automatically. All request/response bodies are JSON; errors use an `error.message_sl` envelope (extractor errors can be plain text). API responses are `Cache-Control: no-store`.

| Method and path | Authentication | Body / behavior |
| --- | --- | --- |
| POST /api/v1/rooms | None | `{ "profile_yaml": "schema_version: 1\n..." }`; returns 201 with room_code, teacher_token, expires_at |
| GET /api/v1/rooms/{code}/profile | None | Returns validated profile object and expires_at |
| POST /api/v1/rooms/{code}/sessions | None | No body; returns 201 with session_id, session_token, generated alias, expires_at |
| PUT /api/v1/rooms/{code}/sessions/{id}/results | Session bearer token | Replaces this session's results; 204 |
| DELETE /api/v1/rooms/{code}/sessions/{id} | Session bearer token | Deletes this session; 204 |
| GET /api/v1/rooms/{code}/results | Teacher bearer token | Sessions (null results until shared), aggregate check counts and expiry |
| DELETE /api/v1/rooms/{code} | Teacher bearer token | Deletes room and all sessions; 204 |
| GET /health | None | Plain text `ok` |

Send secrets only in `Authorization: Bearer <token>`, never query strings. Query strings are rejected. Teacher and session tokens are independent 64-character random hex strings, stored only as SHA-256 hashes and compared in constant time. A room code cannot read results. A session token cannot read teacher results or modify another session, including in another room.

Result payload:

```json
{
  "schema_version": 1,
  "consent": true,
  "results": [
    {"check_id":"python","status":"pass","installed_version":"3.12.0","reason_code":"satisfied"}
  ]
}
```

Provide exactly one result for each check in the room profile. Status must be pass/fail/warning/unknown. `installed_version` and `reason_code` may be omitted or null; versions must be stable semver without build metadata, and reason codes come from the built-in enum. Extra fields (including logs and messages), duplicate IDs, unknown check IDs and missing consent are rejected. A valid one-check profile is required for the payload example above. Results are participant-reported diagnostics, not proof of a trusted device.

Limits: 96 KiB JSON body, 64 KiB YAML profile, 32 checks, 200 sessions per room, 1,000 active rooms, 600 requests/minute per server process. The global limiter deliberately stores no IP address; deploy additional proxy limits against room-code guessing and abusive traffic. A single client can exhaust the global allowance; this is an MVP availability limitation.

Expired rooms return 404 and are removed by access-time cleanup or the 60-second background job. Foreign keys cascade session deletion. 400/422 indicate invalid input, 401 invalid authorization, 409 a full/unavailable room, 413 an oversized body, 429 rate limiting, and 503 room capacity. Unshared sessions appear only to their teacher as generated aliases.
