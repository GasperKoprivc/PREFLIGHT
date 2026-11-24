import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Report } from "./model";
interface Prepared {
  nonce: string;
  action: {
    name_sl: string;
    manager: string;
    package_id: string;
    version: string;
    command: string;
    effects_sl: string;
    permissions_sl: string;
    license_url: string;
  };
}
export default function SafeInstall({
  profileId,
  profileYaml,
  checkId,
  report,
  disabled,
  onBusy,
  onReport,
}: {
  profileId: string;
  profileYaml?: string;
  checkId: string;
  report: Report;
  disabled: boolean;
  onBusy: (b: boolean) => void;
  onReport: (r: Report) => void;
}) {
  const [prepared, setPrepared] = useState<Prepared | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const active = useRef(false);
  useEffect(() => {
    setPrepared(null);
  }, [report]);
  function failure(e: unknown) {
    return typeof e === "object" && e !== null && "message_sl" in e
      ? String(e.message_sl)
      : "Namestitve ni mogoče potrditi. Uporabi ročni vodič.";
  }
  async function prepare() {
    if (active.current) return;
    active.current = true;
    setBusy(true);
    setNotice("");
    try {
      setPrepared(
        await invoke<Prepared>("prepare_fix", {
          profileId,
          profileYaml: profileYaml ?? null,
          checkId,
        }),
      );
    } catch (e) {
      setNotice(failure(e));
    } finally {
      setBusy(false);
      active.current = false;
    }
  }
  async function install() {
    if (!prepared || active.current) return;
    active.current = true;
    setBusy(true);
    onBusy(true);
    setNotice(
      "Namestitev poteka. Počakaj na zaključek in morebitni sistemski poziv Windows (največ 5 minut).",
    );
    const nonce = prepared.nonce;
    setPrepared(null);
    try {
      const result = await invoke<{
        message_sl: string;
        report: Report;
        uncertain: boolean;
      }>("confirm_fix", { nonce, confirmed: true });
      onReport(result.report);
      setNotice(result.message_sl);
    } catch (e) {
      setNotice(failure(e));
    } finally {
      onBusy(false);
      setBusy(false);
      active.current = false;
    }
  }
  return (
    <div className="safe-install">
      <h3>Omejeni samodejni popravek</h3>
      <p>
        Na voljo samo za Git na Windows x64, če zahtevani razpon dovoljuje
        različico 2.56.1. Drugje uporabi ročni vodič.
      </p>
      {!prepared && (
        <button disabled={disabled || busy} onClick={() => void prepare()}>
          Preglej namestitev Git
        </button>
      )}
      {prepared && (
        <div role="region" aria-label="Potrditev namestitve">
          <h4>
            {prepared.action.name_sl} {prepared.action.version}
          </h4>
          <p>
            Upravljalnik: {prepared.action.manager} · paket:{" "}
            {prepared.action.package_id}
          </p>
          <code>{prepared.action.command}</code>
          <p>{prepared.action.effects_sl}</p>
          <p>{prepared.action.permissions_sl}</p>
          <p>
            <a
              href={prepared.action.license_url}
              target="_blank"
              rel="noreferrer"
            >
              Licenca in pogoji Git
            </a>
          </p>
          <p>
            S klikom izrecno dovoliš prikazano spremembo računalnika. Preflight
            ne potrjuje licenčnih ali sistemskih pozivov namesto tebe.
          </p>
          <button disabled={disabled || busy} onClick={() => void install()}>
            Potrdi namestitev
          </button>
          <button disabled={busy} onClick={() => setPrepared(null)}>
            Prekliči
          </button>
        </div>
      )}
      {notice && <p role="status">{notice}</p>}
    </div>
  );
}
