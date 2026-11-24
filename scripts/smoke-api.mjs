import { spawn } from "node:child_process";
import net from "node:net";
import fs from "node:fs";
import path from "node:path";
import assert from "node:assert/strict";

// Isolated ephemeral database; no tokens or participant data are printed.
const listener = net.createServer();
await new Promise((resolve) => listener.listen(0, "127.0.0.1", resolve));
const port = listener.address().port;
await new Promise((resolve) => listener.close(resolve));
const binary = path.resolve(
  "target/debug/preflight-server" +
    (process.platform === "win32" ? ".exe" : ""),
);
const environment = { ...process.env };
delete environment.PREFLIGHT_ALLOWED_ORIGIN;
const child = spawn(binary, [], {
  env: {
    ...environment,
    DATABASE_URL: "sqlite::memory:",
    PREFLIGHT_BIND: `127.0.0.1:${port}`,
    PREFLIGHT_ROOM_TTL_SECONDS: "60",
  },
  stdio: "ignore",
  windowsHide: true,
});
let launchError;
child.on("error", (error) => {
  launchError = error;
});
const base = `http://127.0.0.1:${port}`;
async function request(route, method = "GET", token, body) {
  return fetch(base + route, {
    method,
    headers: {
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(body ? { "Content-Type": "application/json" } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    signal: AbortSignal.timeout(3000),
    redirect: "error",
  });
}
try {
  // The server reads an optional origin; avoid inheriting a production value.
  let ready = false;
  for (let i = 0; i < 50; i++) {
    if (launchError) throw launchError;
    if (child.exitCode !== null)
      throw new Error("Smoke API exited before readiness");
    try {
      if ((await request("/health")).ok) {
        ready = true;
        break;
      }
    } catch {
      /* Starting. */
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert(ready, "Smoke API did not become ready");
  const profile_yaml = fs.readFileSync(
    "crates/preflight-core/tests/fixtures/runtime.yaml",
    "utf8",
  );
  const response = await request("/api/v1/rooms", "POST", undefined, {
    profile_yaml,
  });
  assert.equal(response.status, 201);
  const room = await response.json();
  const route = `/api/v1/rooms/${room.room_code}`;
  const first = await (await request(route + "/sessions", "POST")).json();
  const second = await (await request(route + "/sessions", "POST")).json();
  const payload = {
    schema_version: 1,
    consent: true,
    results: [
      { check_id: "python", status: "pass", installed_version: "3.12.0" },
      { check_id: "git", status: "pass", installed_version: "2.56.1" },
      { check_id: "node", status: "warning", installed_version: "24.0.0" },
    ],
  };
  assert.equal(
    (
      await request(
        `${route}/sessions/${first.session_id}/results`,
        "PUT",
        second.session_token,
        payload,
      )
    ).status,
    401,
  );
  assert.equal(
    (
      await request(
        `${route}/sessions/${first.session_id}/results`,
        "PUT",
        first.session_token,
        payload,
      )
    ).status,
    204,
  );
  assert.equal((await request(route + "/results")).status, 401);
  const data = await (
    await request(route + "/results", "GET", room.teacher_token)
  ).json();
  assert.equal(data.sessions.length, 2);
  assert.equal(data.sessions.filter((s) => s.results === null).length, 1);
  assert.equal(data.aggregate.pass, 2);
  assert.equal(
    (
      await request(
        `${route}/sessions/${first.session_id}`,
        "DELETE",
        first.session_token,
      )
    ).status,
    204,
  );
  assert.equal(
    (await request(route, "DELETE", room.teacher_token)).status,
    204,
  );
  assert.equal((await request(route + "/profile")).status, 404);
  console.log(
    "HTTP smoke passed: two participants, opt-in, authorization, revocation and room deletion.",
  );
} finally {
  child.kill();
}
