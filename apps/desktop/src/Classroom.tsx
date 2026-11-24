import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Profile, Report } from "./model";
interface Room {
  code: string;
  profile: Profile;
  yaml: string;
  expires_at: number;
  shared: boolean;
}
export default function Classroom({
  report,
  running,
  onApply,
}: {
  report: Report | null;
  running: boolean;
  onApply: (p: Profile, y: string) => void;
}) {
  const [server, setServer] = useState("http://127.0.0.1:3001");
  const [code, setCode] = useState("");
  const [room, setRoom] = useState<Room | null>(null);
  const [consent, setConsent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const [shared, setShared] = useState(false);
  useEffect(() => {
    setConsent(false);
  }, [report]);
  async function action(kind: "load" | "share" | "leave") {
    setBusy(true);
    setNotice("");
    try {
      if (kind === "load") {
        const r = await invoke<Room>("room_load", { server, code });
        setRoom(r);
        setConsent(false);
        setShared(false);
        onApply(r.profile, r.yaml);
        setNotice(
          "Profil sobe je izbran. V pregledu okolja ga preveri lokalno. Rezultati še niso deljeni.",
        );
      } else if (kind === "share") {
        await invoke("room_share", { consent });
        setShared(true);
        setConsent(false);
        setNotice(
          "Rezultati so deljeni. Vsaka nadaljnja posodobitev zahteva nov klik in privolitev.",
        );
      } else {
        await invoke("room_leave");
        setRoom(null);
        setShared(false);
        setConsent(false);
        setNotice("Seja je odstranjena. Lokalna diagnostika ostane na voljo.");
      }
    } catch (e) {
      setNotice(
        typeof e === "object" && e !== null && "message_sl" in e
          ? String(e.message_sl)
          : "Povezava ni uspela. Poskusi znova; seja se ohrani za izbris.",
      );
    } finally {
      setBusy(false);
    }
  }
  const payload = report
    ? {
        schema_version: 1,
        consent: true,
        results: report.results.map(
          ({ check_id, status, installed_version, reason_code }) => ({
            check_id,
            status,
            installed_version,
            reason_code,
          }),
        ),
      }
    : null;
  return (
    <section className="info-page">
      <p className="eyebrow">PROSTOVOLJNO DELJENJE</p>
      <h1>Pridruži se delavnici.</h1>
      <p>
        Deljenje je privzeto izključeno. Strežnik vidi IP povezave; pošiljamo
        samo prikazane minimalne rezultate.
      </p>
      <div className="info-card">
        {!room ? (
          <>
            <label>
              Naslov API strežnika
              <input
                value={server}
                onChange={(e) => setServer(e.target.value)}
                disabled={busy || running}
              />
            </label>
            <label>
              Koda sobe
              <input
                maxLength={8}
                value={code}
                onChange={(e) => setCode(e.target.value.toUpperCase())}
                disabled={busy || running}
              />
            </label>
            <button
              disabled={busy || running || code.length !== 8}
              onClick={() => void action("load")}
            >
              Prenesi profil sobe
            </button>
          </>
        ) : (
          <>
            <h2>Soba {room.code}</h2>
            <p>
              {room.profile.name} · poteče{" "}
              {new Date(room.expires_at * 1000).toLocaleString("sl-SI")}
            </p>
            <p>
              {shared
                ? "Rezultati so bili deljeni."
                : "Rezultati še niso deljeni."}
            </p>
            <h3>To se bo delilo</h3>
            {payload ? (
              <pre className="share-preview">
                {JSON.stringify(payload, null, 2)}
              </pre>
            ) : (
              <p>
                Najprej zaženi lokalno preverjanje profila te sobe v pregledu
                okolja.
              </p>
            )}
            <p>
              Rust pred pošiljanjem preveri, ali rezultati pripadajo
              nespremenjenemu profilu sobe. Žetoni ostanejo samo v Rust
              pomnilniku.
            </p>
            <label>
              <input
                type="checkbox"
                checked={consent}
                onChange={(e) => setConsent(e.target.checked)}
                disabled={busy || running}
              />{" "}
              Dovolim pošiljanje teh rezultatov učitelju.
            </label>
            <button
              disabled={!report || !consent || busy || running}
              onClick={() => void action("share")}
            >
              {shared ? "Posodobi deljene rezultate" : "Deli rezultate"}
            </button>
            <button
              disabled={busy || running}
              onClick={() => void action("leave")}
            >
              Izklopi deljenje in izbriši sejo
            </button>
            <p>
              Ob napaki izbrisa poskusi znova. Pred zaprtjem aplikacije izbriši
              sejo; sicer ostane na strežniku do poteka sobe (največ 24 ur).
              Zaprtje aplikacije izgubi lokalni žeton.
            </p>
          </>
        )}
      </div>
      {notice && <p role="status">{notice}</p>}
    </section>
  );
}
