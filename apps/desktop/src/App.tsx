import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  type Profile,
  type Report,
  type Status,
  statusLabels,
  toolNames,
} from "./model";

type Page = "checks" | "privacy" | "settings";
type Theme = "system" | "light" | "dark";
const symbols: Record<Status, string> = {
  pass: "✓",
  fail: "!",
  warning: "!",
  unknown: "?",
};
function initialTheme(): Theme {
  try {
    const saved = localStorage.getItem("preflight-theme");
    return saved === "light" || saved === "dark" ? saved : "system";
  } catch {
    return "system";
  }
}
function message(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message_sl" in error &&
    typeof error.message_sl === "string"
  )
    return error.message_sl;
  return "Lokalno jedro ni dosegljivo. Odpri namizno aplikacijo Preflight in poskusi znova.";
}
export default function App() {
  const [page, setPage] = useState<Page>("checks");
  const [theme, setTheme] = useState<Theme>(initialTheme);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [selected, setSelected] = useState("");
  const [loading, setLoading] = useState(true);
  const [running, setRunning] = useState(false);
  const [report, setReport] = useState<Report | null>(null);
  const [error, setError] = useState("");
  const [lastRun, setLastRun] = useState("");
  const [loadAttempt, setLoadAttempt] = useState(0);
  const [version, setVersion] = useState("");
  const resultHeading = useRef<HTMLHeadingElement>(null);
  const inFlight = useRef(false);
  const profile = profiles.find((p) => p.id === selected);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem("preflight-theme", theme);
    } catch {
      /* Theme still works without storage. */
    }
  }, [theme]);
  useEffect(() => {
    let active = true;
    invoke<Profile[]>("list_profiles")
      .then((items) => {
        if (!active) return;
        setProfiles(items);
        setSelected(items[0]?.id ?? "");
        setLoading(false);
      })
      .catch((e) => {
        if (active) {
          setError(message(e));
          setLoading(false);
        }
      });
    invoke<{ name: string; version: string }>("get_app_info")
      .then((info) => {
        if (active) setVersion(info.version);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [loadAttempt]);
  useEffect(() => {
    if (report) resultHeading.current?.focus();
  }, [report]);

  async function run() {
    if (!profile || inFlight.current) return;
    inFlight.current = true;
    setRunning(true);
    setError("");
    setReport(null);
    setLastRun("");
    try {
      const result = await invoke<Report>("run_checks", {
        profileId: profile.id,
      });
      setReport(result);
      setLastRun(
        new Date().toLocaleTimeString("sl-SI", {
          hour: "2-digit",
          minute: "2-digit",
        }),
      );
    } catch (e) {
      setError(message(e));
    } finally {
      inFlight.current = false;
      setRunning(false);
    }
  }

  return (
    <div className="shell">
      <a className="skip-link" href="#content">
        Preskoči na vsebino
      </a>
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-icon" aria-hidden="true">
            ↗
          </span>{" "}
          preflight<span className="brand-dot">.</span>
        </div>
        <p className="nav-label">TVOJ DELOVNI PROSTOR</p>
        <nav aria-label="Glavna navigacija">
          <button
            className={page === "checks" ? "nav-item active" : "nav-item"}
            aria-current={page === "checks" ? "page" : undefined}
            onClick={() => setPage("checks")}
          >
            <span aria-hidden="true">▦</span> Pregled okolja
          </button>
          <button
            className={page === "privacy" ? "nav-item active" : "nav-item"}
            aria-current={page === "privacy" ? "page" : undefined}
            onClick={() => setPage("privacy")}
          >
            <span aria-hidden="true">◇</span> Zasebnost
          </button>
          <button
            className={page === "settings" ? "nav-item active" : "nav-item"}
            aria-current={page === "settings" ? "page" : undefined}
            onClick={() => setPage("settings")}
          >
            <span aria-hidden="true">☷</span> Nastavitve
          </button>
        </nav>
        <div className="sidebar-note">
          <span className="offline-dot" /> Lokalno. Zasebno.
          <p>
            Brez računa.
            <br />
            Brez pošiljanja rezultatov.
          </p>
        </div>
        <div className="version">
          PREFLIGHT {version || "—"}
          <span>OFFLINE FIRST</span>
        </div>
      </aside>
      <main id="content">
        <header className="topbar">
          <span>PRIPRAVA NA DELAVNICO</span>
          <span className="local-badge">
            <span className="offline-dot" /> Na tvojem računalniku
          </span>
        </header>
        {page === "checks" && (
          <>
            <div className="page-heading">
              <div>
                <p className="eyebrow">MANJ ZAPLETOV. VEČ USTVARJANJA.</p>
                <h1>Najprej preverimo okolje.</h1>
                <p className="muted">
                  Vse, kar potrebuješ za dober začetek delavnice.
                </p>
              </div>
              <span className="heading-mark" aria-hidden="true">
                ↗
              </span>
            </div>
            <section className="profile-card" aria-labelledby="profile-heading">
              <div className="step-number" aria-hidden="true">
                01
              </div>
              <div className="profile-field">
                <label id="profile-heading" htmlFor="profile">
                  Izberi delavnico
                </label>
                <select
                  id="profile"
                  value={selected}
                  disabled={loading || running}
                  onChange={(e) => {
                    setSelected(e.target.value);
                    setReport(null);
                    setError("");
                    setLastRun("");
                  }}
                >
                  <option value="">Izberi profil …</option>
                  {profiles.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
                <span className="field-note">
                  {profile
                    ? `${profile.checks.length} lokalna preverjanja · brez spreminjanja sistema`
                    : "Izberi zahteve za svoje razvojno okolje."}
                </span>
              </div>
              <button
                className="primary"
                onClick={() => void run()}
                disabled={!profile || loading || running}
              >
                {running
                  ? "Preverjanje poteka …"
                  : report
                    ? "Ponovno preveri"
                    : "Zaženi preverjanje"}
                <span aria-hidden="true">{running ? "◌" : "→"}</span>
              </button>
            </section>
            {error && (
              <div className="error" role="alert">
                <strong>Preverjanje ni uspelo.</strong>
                <p>{error}</p>
                {profiles.length === 0 && (
                  <button
                    onClick={() => {
                      setLoading(true);
                      setError("");
                      setLoadAttempt((x) => x + 1);
                    }}
                  >
                    Poskusi znova
                  </button>
                )}
              </div>
            )}
            {loading && (
              <p role="status" className="empty">
                Nalaganje lokalnih profilov …
              </p>
            )}
            {!loading && !profile && !error && (
              <div className="empty">
                <h2>Začni z izbiro profila.</h2>
                <p>
                  {profiles.length
                    ? "Zgoraj izberi delavnico, da prikažemo njene zahteve."
                    : "Trenutno ni na voljo noben profil."}
                </p>
              </div>
            )}
            {profile && (
              <>
                <section
                  className={`summary ${report ? (report.summary.ready ? "ready" : "attention") : ""}`}
                  aria-live="polite"
                  aria-atomic="true"
                >
                  <div className="summary-symbol" aria-hidden="true">
                    {running
                      ? "◌"
                      : report
                        ? report.summary.ready
                          ? "✓"
                          : "!"
                        : "◎"}
                  </div>
                  <div>
                    <h2 ref={resultHeading} tabIndex={-1}>
                      {running
                        ? "Preverjamo tvoje okolje …"
                        : report
                          ? report.summary.ready
                            ? "Obvezne zahteve so izpolnjene."
                            : "Nekaj stvari še potrebuje pozornost."
                          : "Tvoj prvi pregled je pripravljen."}
                    </h2>
                    <p>
                      {running
                        ? "Orodja preverjamo lokalno. Vsak ukaz ima časovno omejitev 5 sekund."
                        : report
                          ? `${report.summary.pass} v redu · ${report.summary.fail} napak · ${report.summary.warning} opozoril · ${report.summary.unknown} neznanih`
                          : "Zaženi preverjanje in izvedi, kaj že deluje ter kaj je treba urediti."}
                    </p>
                  </div>
                  {lastRun && (
                    <span className="last-run">Preverjeno ob {lastRun}</span>
                  )}
                </section>
                <div className="section-title">
                  <h2>
                    Zahteve delavnice <span>{profile.checks.length}</span>
                  </h2>
                  <span>Klikni za podrobnosti</span>
                </div>
                <div className="checks" aria-busy={running}>
                  {profile.checks.map((check) => {
                    const result = report?.results.find(
                      (r) => r.check_id === check.id,
                    );
                    return (
                      <details
                        key={`${profile.id}-${check.id}`}
                        className="check-row"
                      >
                        <summary>
                          <span
                            className={`tool-icon ${check.tool}`}
                            aria-hidden="true"
                          >
                            {check.tool === "python"
                              ? "Py"
                              : check.tool === "node"
                                ? "JS"
                                : check.tool === "git"
                                  ? "Gt"
                                  : "Jv"}
                          </span>
                          <span className="tool-description">
                            <strong>{toolNames[check.tool]}</strong>
                            <span>
                              {check.required ? "Obvezno" : "Neobvezno"} ·
                              zahteva {check.version}
                            </span>
                          </span>
                          <span className="installed">
                            {result?.installed_version ?? "—"}
                            <small>nameščeno</small>
                          </span>
                          <span
                            className={`status ${result?.status ?? "pending"}`}
                          >
                            <span aria-hidden="true">
                              {result ? symbols[result.status] : "○"}
                            </span>
                            {result
                              ? statusLabels[result.status]
                              : running
                                ? "V preverjanju"
                                : "Še ni preverjeno"}
                          </span>
                          <span className="chevron" aria-hidden="true">
                            ⌄
                          </span>
                        </summary>
                        <div className="check-detail">
                          <h3>
                            {result ? result.message_sl : "Kaj preverimo?"}
                          </h3>
                          <p>
                            {result
                              ? result.suggestion_sl
                              : `Preverimo, ali je ${toolNames[check.tool]} dosegljiv in njegova različica ustreza zahtevi ${check.version}.`}
                          </p>
                          {result?.reason_code === "timeout" && (
                            <p>
                              Časovna omejitev ne pomeni, da program manjka.
                              Poskusi s ponovnim preverjanjem.
                            </p>
                          )}
                        </div>
                      </details>
                    );
                  })}
                </div>
                <div className="privacy-note">
                  <span aria-hidden="true">◇</span>
                  <p>
                    <strong>Rezultati ostanejo na tvojem računalniku.</strong>{" "}
                    Preflight ne namešča programov in ne spreminja tvojih
                    nastavitev.
                  </p>
                </div>
              </>
            )}
          </>
        )}
        {page === "privacy" && (
          <section className="info-page">
            <p className="eyebrow">TI ODLOČAŠ O SVOJIH PODATKIH</p>
            <h1>Lokalno pomeni tvoje.</h1>
            <p>Rezultati ostanejo na tvojem računalniku.</p>
            <div className="info-card">
              <h2>Deljenje je izključeno.</h2>
              <p>
                Ta različica ne pošilja rezultatov učitelju ali strežniku.
                Učiteljske sobe in prostovoljno deljenje še niso na voljo.
              </p>
              <h2>Kaj preverjamo?</h2>
              <p>
                Samo prisotnost in različice orodij iz izbranega profila. Surovi
                izhodi se ne prikazujejo ali shranjujejo v rezultate. Ne zbiramo
                uporabniških imen, strojnih ID-jev ali osebnih datotek.
              </p>
              <h2>Brez sprememb sistema.</h2>
              <p>
                Preverjanje ničesar ne namešča. Priporočila lahko prebereš v
                podrobnostih posameznega rezultata.
              </p>
            </div>
          </section>
        )}
        {page === "settings" && (
          <section className="info-page">
            <p className="eyebrow">PO TVOJE</p>
            <h1>Mirno delovno okolje.</h1>
            <div className="info-card">
              <h2>Videz aplikacije</h2>
              <label htmlFor="theme">Barvna tema</label>
              <select
                id="theme"
                value={theme}
                onChange={(e) => setTheme(e.target.value as Theme)}
              >
                <option value="system">Kot sistem</option>
                <option value="light">Svetla</option>
                <option value="dark">Temna</option>
              </select>
              <p>
                Izbira teme se shrani samo na tem računalniku. Rezultati
                preverjanja ostanejo v pomnilniku do zaprtja aplikacije.
              </p>
            </div>
          </section>
        )}
        <footer>
          <span>Priprava, ki ti pusti več časa za učenje.</span>
          <span>PREVERI. UREDI. USTVARJAJ.</span>
        </footer>
      </main>
    </div>
  );
}
