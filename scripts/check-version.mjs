import fs from "node:fs";
const root = JSON.parse(fs.readFileSync("package.json", "utf8"));
const desktop = JSON.parse(
  fs.readFileSync("apps/desktop/package.json", "utf8"),
);
const config = JSON.parse(
  fs.readFileSync("apps/desktop/src-tauri/tauri.conf.json", "utf8"),
);
const cargo = fs
  .readFileSync("Cargo.toml", "utf8")
  .match(/\[workspace.package\][\s\S]*?version\s*=\s*"([^"]+)"/);
if (
  !cargo ||
  [root.version, desktop.version, config.version, cargo[1]].some(
    (v) => v !== "0.1.0",
  )
)
  throw new Error("Versions must all be 0.1.0 before tagging");
console.log("Version alignment: 0.1.0");
