import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { type Profile } from "./model";
interface Props {
  profile?: Profile;
  yaml?: string;
  disabled: boolean;
  onApply: (profile: Profile, yaml: string) => void;
}
const template = `schema_version: 1\nid: moja-delavnica\nname: "Moja delavnica"\nchecks:\n  - id: python\n    type: runtime\n    tool: python\n    version: ">=3.11.0"\n    required: true\n`;
function safeMessage(error: unknown): string {
  return typeof error === "object" &&
    error !== null &&
    "message_sl" in error &&
    typeof error.message_sl === "string"
    ? error.message_sl
    : "Profila ni mogoče obdelati. Preveri povezavo z namiznim jedrom.";
}
export default function ProfileEditor({
  profile,
  yaml,
  disabled,
  onApply,
}: Props) {
  const [text, setText] = useState(template);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  async function loadSelected() {
    if (!profile) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setText(
        await invoke<string>("get_profile_yaml", {
          profileId: profile.id,
          profileYaml: yaml ?? null,
        }),
      );
    } catch (e) {
      setError(safeMessage(e));
    } finally {
      setBusy(false);
    }
  }
  async function apply(exportFile: boolean) {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      if (new TextEncoder().encode(text).length > 65536) {
        setError("Profil je prevelik. Največja velikost je 64 KiB.");
        return;
      }
      const validated = await invoke<Profile>("validate_profile", {
        yaml: text,
      });
      if (exportFile) {
        const saved = await invoke<boolean>("export_profile", {
          profileId: validated.id,
          profileYaml: text,
        });
        setNotice(
          saved
            ? "Profil je shranjen v izbrano datoteko."
            : "Shranjevanje je preklicano.",
        );
      } else {
        onApply(validated, text);
        setNotice("Profil je preverjen in izbran za naslednje preverjanje.");
      }
    } catch (e) {
      setError(safeMessage(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="info-page editor">
      <p className="eyebrow">ZA TVOJO DELAVNICO</p>
      <h1>En profil. Jasne zahteve.</h1>
      <p>
        Uvozi ali uredi lokalni YAML. Pred uporabo ga preveri Rust jedro.
        Spremembe z enakim ID-jem zamenjajo profil samo v tej seji; za ohranitev
        izvozi datoteko.
      </p>
      <div className="editor-toolbar">
        <button
          disabled={disabled || busy || !profile}
          onClick={() => void loadSelected()}
        >
          Uredi izbrani profil
        </button>
        <label className="file-label">
          Uvozi YAML
          <input
            type="file"
            accept=".yaml,.yml"
            disabled={disabled || busy}
            aria-label="Uvozi YAML"
            onChange={async (e) => {
              const file = e.target.files?.[0];
              if (!file) return;
              setError("");
              setNotice("");
              if (file.size > 65536) {
                setError("Profil je prevelik. Največja velikost je 64 KiB.");
                e.target.value = "";
                return;
              }
              try {
                setText(await file.text());
                setNotice("Datoteka je odprta. Preveri vsebino in jo potrdi.");
              } catch {
                setError("Datoteke ni mogoče prebrati.");
              }
              e.target.value = "";
            }}
          />
        </label>
      </div>
      <label htmlFor="yaml">Vsebina profila YAML</label>
      <textarea
        id="yaml"
        spellCheck={false}
        value={text}
        disabled={disabled || busy}
        onChange={(e) => {
          setText(e.target.value);
          setError("");
          setNotice("");
        }}
        aria-describedby="yaml-help"
      />
      <p id="yaml-help" className="editor-help">
        Dovoljene vrste: runtime, vscode_extension, disk_space, https. Omejitve:
        64 KiB in 32 preverjanj. Brez ukazov, skript, YAML tagov ali aliasov.
        HTTPS naslovi morajo biti javni.
      </p>
      {error && (
        <div className="error" role="alert">
          {error}
        </div>
      )}
      {notice && <p role="status">{notice}</p>}
      <div className="editor-toolbar">
        <button
          className="primary"
          disabled={disabled || busy}
          onClick={() => void apply(false)}
        >
          Preveri in uporabi
        </button>
        <button disabled={disabled || busy} onClick={() => void apply(true)}>
          Izvozi YAML
        </button>
      </div>
    </section>
  );
}
