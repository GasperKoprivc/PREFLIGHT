import { useState } from "react";
import { guidance, type OperatingSystem } from "./guidance";
import type { Check, CheckResult } from "./model";
export default function GuidedFix({
  check,
  result,
  platform,
  onRecheck,
  disabled,
}: {
  check: Check;
  result: CheckResult;
  platform: string;
  onRecheck: () => void;
  disabled: boolean;
}) {
  const [chosen, setChosen] = useState<OperatingSystem | null>(null);
  const [notice, setNotice] = useState("");
  const os =
    chosen ??
    (platform === "windows" || platform === "macos" || platform === "linux"
      ? platform
      : "other");
  const guide = guidance(check, result, os);
  return (
    <div className="guided-fix">
      <h3>{guide.title}</h3>
      <label>
        Navodila za sistem
        <select
          value={os}
          onChange={(e) => {
            setChosen(e.target.value as OperatingSystem);
            setNotice("");
          }}
        >
          <option value="windows">Windows</option>
          <option value="macos">macOS</option>
          <option value="linux">Linux</option>
          <option value="other">Drug ali neznan sistem</option>
        </select>
      </label>
      <ol>
        {guide.steps.map((step, i) => (
          <li key={i}>{step}</li>
        ))}
      </ol>
      {guide.links.map((link) => (
        <p key={link.url}>
          <a href={link.url} target="_blank" rel="noreferrer">
            {link.label} ↗
          </a>
        </p>
      ))}
      {guide.commands.map((item) => (
        <div className="manual-command" key={item.command}>
          <p>{item.label}</p>
          <code>{item.command}</code>
          <button
            onClick={async () => {
              try {
                await navigator.clipboard.writeText(item.command);
                setNotice("Ukaz je kopiran. Zaženeš ga sam v terminalu.");
              } catch {
                setNotice(
                  "Kopiranje ni na voljo. Označi in ročno kopiraj prikazani ukaz.",
                );
              }
            }}
          >
            Kopiraj ukaz
          </button>
        </div>
      ))}
      {guide.commands.length > 0 && (
        <p>
          Ukazi se ne izvajajo samodejno. Pred ročno namestitvijo preberi ukaz;
          lahko prenese programsko opremo ali zahteva dodatna dovoljenja.
          Različica mora ustrezati zahtevi profila.
        </p>
      )}
      <button disabled={disabled} onClick={onRecheck}>
        Ponovno preveri to zahtevo
      </button>
      {notice && <p role="status">{notice}</p>}
    </div>
  );
}
