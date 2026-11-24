import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  type Profile,
  type Report,
  type Status,
  statusLabels,
  checkName,
  checkRequirement,
  checkIcon,
} from "./model";

import SafeInstall from "./SafeInstall";
import GuidedFix from "./GuidedFix";
import Classroom from "./Classroom";
import ProfileEditor from "./ProfileEditor";
type Page = "checks" | "privacy" | "settings" | "profiles" | "room";
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
  const [platform, setPlatform] = useState("other");
  const [customYaml, setCustomYaml] = useState<Record<string, string>>({});
  const [allowNetwork, setAllowNetwork] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [exportNotice, setExportNotice] = useState("");
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
    invoke<{ name: string; version: string; platform?: string }>("get_app_info")
      .then((info) => {
        if (active) {
          setVersion(info.version);
          setPlatform(info.platform ?? "other");
        }
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [loadAttempt]);
  useEffect(() => {
    if (report) resultHeading.current?.focus();
  }, [report]);

  async function run(checkId?: string) {
    if (!profile || inFlight.current) return;
    inFlight.current = true;
    setRunning(true);
    setError("");
    setReport(null);
    setLastRun("");
    setExportNotice("");
    try {
      const result = await invoke<Report>("run_checks", {
        profileId: profile.id,
        profileYaml: customYaml[profile.id] ?? null,
        allowNetwork,
        ...(checkId ? { checkId } : {}),
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

  async function exportResults(format: "json" | "text") {
    setExporting(true);
    setExportNotice("");
    try {
      const saved = await invoke<boolean>("export_report", { format });
      setExportNotice(
        saved ? "Povzetek je shranjen lokalno." : "Shranjevanje je preklicano.",
      );
    } catch (e) {
      setError(message(e));
    } finally {
      setExporting(false);
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
          <button
            className={page === "profiles" ? "nav-item active" : "nav-item"}
            aria-current={page === "profiles" ? "page" : undefined}
            onClick={() => setPage("profiles")}
          >
            <span aria-hidden="true">≡</span> Profili
          </button>
          <button
            className={page === "room" ? "nav-item active" : "nav-item"}
            onClick={() => setPage("room")}
          >
            <span aria-hidden="true">◇</span> Soba
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
                    setAllowNetwork(false);
                    setExportNotice("");
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
                    ? `${profile.checks.length} preverjanj · brez spreminjanja sistema`
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
            {profile?.checks.some((c) => c.type === "https") && (
              <div className="network-choice">
                <label>
                  <input
                    type="checkbox"
                    checked={allowNetwork}
                    disabled={running}
                    onChange={(e) => {
                      setAllowNetwork(e.target.checked);
                      setReport(null);
                      setLastRun("");
                      setExportNotice("");
                    }}
                  />{" "}
                  Dovoli HTTPS preverjanja za ta profil
                </label>
                <p>
                  Ob vklopu stopimo v stik z naslovi v profilu. Ciljni strežnik
                  vidi tvoj IP naslov; rezultatov mu ne pošiljamo. Brez kljukice
                  ostane preverjanje lokalno.
                </p>
              </div>
            )}
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
                        ? "Vsak ukaz ima omejitev 5 sekund; HTTPS preverjanje največ 12 sekund."
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
                            {checkIcon(check)}
                          </span>
                          <span className="tool-description">
                            <strong>{checkName(check)}</strong>
                            <span>
                              {check.required ? "Obvezno" : "Neobvezno"} ·
                              zahteva {checkRequirement(check)}
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
                              : `Preverjanje: ${checkName(check)}. Zahteva: ${checkRequirement(check)}.`}
                          </p>
                          {result && (
                            <GuidedFix
                              check={check}
                              result={result}
                              platform={platform}
                              disabled={running}
                              onRecheck={() => void run(check.id)}
                            />
                          )}
                          {result &&
                            platform === "windows" &&
                            check.tool === "git" &&
                            ["not_found", "version_mismatch"].includes(
                              result.reason_code,
                            ) &&
                            report && (
                              <SafeInstall
                                profileId={profile.id}
                                profileYaml={customYaml[profile.id]}
                                checkId={check.id}
                                report={report}
                                disabled={running}
                                onBusy={setRunning}
                                onReport={(r) => {
                                  setReport(r);
                                  setLastRun(
                                    new Date().toLocaleTimeString("sl-SI"),
                                  );
                                }}
                              />
                            )}
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
                {report && (
                  <div className="export-controls">
                    <span>Lokalni povzetek</span>
                    <button
                      disabled={exporting}
                      onClick={() => void exportResults("json")}
                    >
                      Izvozi JSON
                    </button>
                    <button
                      disabled={exporting}
                      onClick={() => void exportResults("text")}
                    >
                      Izvozi besedilo
                    </button>
                  </div>
                )}
                {exportNotice && <p role="status">{exportNotice}</p>}
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
        {page === "profiles" && (
          <ProfileEditor
            profile={profile}
            yaml={profile ? customYaml[profile.id] : undefined}
            disabled={running}
            onApply={(p, yaml) => {
              setProfiles((items) => [
                ...items.filter((x) => x.id !== p.id),
                p,
              ]);
              setCustomYaml((items) => ({ ...items, [p.id]: yaml }));
              setSelected(p.id);
              setReport(null);
              setLastRun("");
              setError("");
              setAllowNetwork(false);
              setExportNotice("");
            }}
          />
        )}
        <div hidden={page !== "room"}>
          <Classroom
            report={report}
            running={running}
            onApply={(p, y) => {
              setProfiles((items) => [
                ...items.filter((x) => x.id !== p.id),
                p,
              ]);
              setCustomYaml((items) => ({ ...items, [p.id]: y }));
              setSelected(p.id);
              setReport(null);
              setLastRun("");
              setAllowNetwork(false);
            }}
          />
        </div>
        {page === "privacy" && (
          <section className="info-page">
            <p className="eyebrow">TI ODLOČAŠ O SVOJIH PODATKIH</p>
            <h1>Lokalno pomeni tvoje.</h1>
            <p>Rezultati ostanejo na tvojem računalniku.</p>
            <div className="info-card">
              <h2>Deljenje je izključeno.</h2>
              <p>
                Lokalni pregled ne pošilja rezultatov učitelju ali strežniku.
                Deljenje je na voljo v pogledu Soba samo po tvoji izrecni
                privolitvi.
              </p>
              <h2>Spletni testi samo po izbiri.</h2>
              <p>
                Če dovoliš HTTPS preverjanja, se povežemo z javnimi naslovi iz
                profila. Ti strežniki vidijo tvoj IP naslov. Samih rezultatov ne
                pošiljamo. Profili in izvozi lahko vsebujejo tvoje vnesene
                ID-je; vanje ne zapisuj osebnih podatkov.
              </p>
              <h2>Kaj preverjamo?</h2>
              <p>
                Prisotnost in različice orodij, zahtevane razširitve, prosti
                prostor ter izbrane HTTPS povezave. Surovi izhodi se ne
                prikazujejo ali shranjujejo v rezultate. Ne zbiramo uporabniških
                imen, strojnih ID-jev ali osebnih datotek.
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
