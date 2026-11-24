import { useEffect, useState } from "react";
import {
  api,
  ApiError,
  restoreRoom,
  labels,
  type TeacherRoom,
  type RoomResults,
} from "./api";
const initialYaml = `schema_version: 1\nid: python-workshop\nname: "Python delavnica"\nchecks:\n  - id: python\n    type: runtime\n    tool: python\n    version: ">=3.11.0"\n    required: true\n`;
export default function App() {
  const [room, setRoom] = useState<TeacherRoom | null>(restoreRoom);
  const [yaml, setYaml] = useState(initialYaml);
  const [results, setResults] = useState<RoomResults | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [last, setLast] = useState("");
  const [filter, setFilter] = useState("");
  const [paused, setPaused] = useState(false);
  const [tick, setTick] = useState(0);
  const [saved, setSaved] = useState(true);
  const [showSecret, setShowSecret] = useState(false);
  const invite = new URLSearchParams(location.search).get("room");
  useEffect(() => {
    try {
      if (room) sessionStorage.setItem("preflight-room", JSON.stringify(room));
      else sessionStorage.removeItem("preflight-room");
      setSaved(true);
    } catch {
      setSaved(false);
    }
  }, [room]);
  useEffect(() => {
    if (!room || paused) return;
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      if (room!.expires_at > 0 && Date.now() >= room!.expires_at * 1000) {
        setPaused(true);
        setError("Soba je potekla.");
        return;
      }
      try {
        const data = await api<RoomResults>(
          `/rooms/${room!.room_code}/results`,
          "GET",
          room!.teacher_token,
        );
        if (active) {
          setResults(data);
          setLast(new Date().toLocaleTimeString("sl-SI"));
          setError("");
        }
      } catch (e) {
        if (active) {
          setError(
            e instanceof ApiError
              ? e.message
              : "Strežnik ni dosegljiv. Prikazani so zadnji znani podatki.",
          );
          if (e instanceof ApiError && [401, 404].includes(e.status)) {
            setPaused(true);
            return;
          }
        }
      }
      if (active) timer = setTimeout(() => void poll(), 5000);
    }
    void poll();
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [room, paused, tick]);
  async function create() {
    setBusy(true);
    setError("");
    try {
      const created = await api<TeacherRoom>("/rooms", "POST", undefined, {
        profile_yaml: yaml,
      });
      setRoom(created);
      setResults(null);
      setPaused(false);
      setLast("");
    } catch (e) {
      setError(
        e instanceof ApiError
          ? e.message
          : "Sobe ni mogoče ustvariti. Preveri povezavo.",
      );
    } finally {
      setBusy(false);
    }
  }
  async function remove() {
    if (!room) return;
    setBusy(true);
    try {
      await api(`/rooms/${room.room_code}`, "DELETE", room.teacher_token);
      setRoom(null);
      setResults(null);
      setError("");
    } catch (e) {
      if (e instanceof ApiError && e.status === 404) {
        setRoom(null);
        setResults(null);
        setError("");
      } else {
        setError(
          e instanceof ApiError
            ? e.message
            : "Brisanje ni uspelo. Poskusi znova.",
        );
      }
    } finally {
      setBusy(false);
    }
  }
  const checkIds = [
    ...new Set(
      results?.sessions.flatMap(
        (s) =>
          s.results
            ?.filter((r) => r.status !== "pass")
            .map((r) => r.check_id) ?? [],
      ) ?? [],
    ),
  ];
  return (
    <main>
      <header>
        <strong>preflight.</strong>
        <span>ZA UČITELJE</span>
      </header>
      <h1>Pripravljeni skupaj.</h1>
      <p>Prostovoljno deljeni rezultati brez imen in računov.</p>
      {invite && /^[A-HJ-NP-Z2-9]{8}$/.test(invite) && (
        <aside>
          Koda delavnice: <strong>{invite}</strong>. V namizni aplikaciji odpri
          »Soba« in vnesi kodo ter naslov tega strežnika: {location.origin}.
        </aside>
      )}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {!room ? (
        <section>
          <h2>Ustvari delavnico</h2>
          <label htmlFor="profile">Profil YAML</label>
          <textarea
            id="profile"
            value={yaml}
            onChange={(e) => setYaml(e.target.value)}
            disabled={busy}
          />
          <p>
            Strežnik profil validira. Vanj ne vpisuj imen ali drugih osebnih
            podatkov.
          </p>
          <button
            disabled={busy || new TextEncoder().encode(yaml).length > 65536}
            onClick={() => void create()}
          >
            {busy ? "Ustvarjanje …" : "Ustvari sobo"}
          </button>
          <details>
            <summary>Obnovi nadzor sobe</summary>
            <Restore
              onRestore={(r) => {
                setRoom(r);
                setResults(null);
                setPaused(false);
                setError("");
              }}
            />
          </details>
        </section>
      ) : (
        <>
          <section className="room-heading">
            <div>
              <p>KODA SOBE</p>
              <h2 className="code">{room.room_code}</h2>
              <label>
                Povezava za udeležence
                <input
                  readOnly
                  value={`${location.origin}/?room=${room.room_code}`}
                />
              </label>
              <p>
                Poteče:{" "}
                {new Date(
                  (results?.expires_at ?? room.expires_at) * 1000,
                ).toLocaleString("sl-SI")}
              </p>
            </div>
            <div>
              <label>
                Učiteljska skrivnost
                <input
                  type={showSecret ? "text" : "password"}
                  readOnly
                  value={room.teacher_token}
                />
              </label>
              <button
                className="secondary"
                onClick={() => setShowSecret((v) => !v)}
              >
                {showSecret
                  ? "Skrij skrivnost"
                  : "Pokaži za varno shranjevanje"}
              </button>
              <p>
                Skrivnost se hrani v tej seji brskalnika in preživi osvežitev.
                Pred zaprtjem zavihka jo shrani na varno; nikoli je ne pošiljaj
                udeležencem.
              </p>
              {!saved && (
                <p role="alert">
                  Shranjevanje seje ni na voljo. Pred osvežitvijo varno shrani
                  skrivnost.
                </p>
              )}
            </div>
          </section>
          <div className="toolbar">
            <p role="status">
              {last ? `Zadnja osvežitev: ${last}` : "Nalaganje rezultatov …"} ·{" "}
              {paused ? "Osveževanje ustavljeno" : "Osvežitev vsakih 5 sekund"}
            </p>
            <button
              className="secondary"
              onClick={() => {
                setPaused(false);
                setTick((t) => t + 1);
              }}
            >
              Osveži
            </button>
            <label>
              Težavna preverba
              <select
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
              >
                <option value="">Vse</option>
                {checkIds.map((id) => (
                  <option key={id}>{id}</option>
                ))}
              </select>
            </label>
          </div>
          <section>
            <h2>{results?.sessions.length ?? 0} udeležencev</h2>
            <div className="stats">
              {Object.entries(labels).map(([key, label]) => (
                <span key={key}>
                  {label}: <strong>{results?.aggregate[key] ?? 0}</strong>
                </span>
              ))}
            </div>
            <p>
              Števila predstavljajo posamezne preverbe. Rezultati so poročila
              udeležencev.
            </p>
            {results?.sessions.length === 0 && (
              <p>
                Soba je prazna. Deli kodo; rezultati se pojavijo šele po
                soglasju.
              </p>
            )}
            {results?.sessions
              .filter(
                (s) =>
                  !filter ||
                  s.results?.some(
                    (r) => r.check_id === filter && r.status !== "pass",
                  ),
              )
              .map((s) => (
                <article key={s.session_id}>
                  <h3>{s.alias}</h3>
                  {s.results ? (
                    <ul>
                      {s.results
                        .filter((r) => !filter || r.check_id === filter)
                        .map((r) => (
                          <li key={r.check_id}>
                            <strong>{r.check_id}</strong> — {labels[r.status]}{" "}
                            {r.installed_version && `(${r.installed_version})`}
                          </li>
                        ))}
                    </ul>
                  ) : (
                    <p>Rezultati še niso deljeni.</p>
                  )}
                </article>
              ))}
          </section>
          <details>
            <summary>Popravi učiteljski dostop</summary>
            <Restore
              onRestore={(r) => {
                setRoom(r);
                setResults(null);
                setPaused(false);
                setError("");
              }}
            />
          </details>
          <details>
            <summary>Zaključi delavnico</summary>
            <p>Izbris odstrani sobo in vse njene seje.</p>
            <button disabled={busy} onClick={() => void remove()}>
              Izbriši sobo in rezultate
            </button>
          </details>
        </>
      )}
      <footer>
        Rezultati se odstranijo ob poteku sobe. Udeleženec lahko svojo sejo
        izbriše.{" "}
        <a
          href="https://github.com/GasperKoprivc/PREFLIGHT"
          target="_blank"
          rel="noreferrer"
        >
          O projektu
        </a>
      </footer>
    </main>
  );
}
function Restore({ onRestore }: { onRestore: (r: TeacherRoom) => void }) {
  const [code, setCode] = useState("");
  const [token, setToken] = useState("");
  return (
    <div>
      <label>
        Koda sobe
        <input
          value={code}
          onChange={(e) => setCode(e.target.value.toUpperCase())}
          maxLength={8}
        />
      </label>
      <label>
        Učiteljska skrivnost
        <input
          type="password"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          autoComplete="off"
        />
      </label>
      <button
        disabled={
          !/^[A-HJ-NP-Z2-9]{8}$/.test(code) || !/^[a-f0-9]{64}$/.test(token)
        }
        onClick={() =>
          onRestore({ room_code: code, teacher_token: token, expires_at: 0 })
        }
      >
        Obnovi nadzor
      </button>
    </div>
  );
}
