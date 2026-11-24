CREATE TABLE rooms (
 code TEXT PRIMARY KEY,
 teacher_hash TEXT NOT NULL,
 profile_yaml TEXT NOT NULL,
 created_at INTEGER NOT NULL,
 expires_at INTEGER NOT NULL
);
CREATE INDEX rooms_expiry ON rooms(expires_at);
CREATE TABLE sessions (
 id TEXT PRIMARY KEY,
 room_code TEXT NOT NULL REFERENCES rooms(code) ON DELETE CASCADE,
 token_hash TEXT NOT NULL,
 alias TEXT NOT NULL,
 results_json TEXT,
 updated_at INTEGER NOT NULL
);
CREATE INDEX sessions_room ON sessions(room_code);
