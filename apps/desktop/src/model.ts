export type Status = "pass" | "fail" | "warning" | "unknown";
export interface Check {
  id: string;
  type: "runtime" | "vscode_extension" | "disk_space" | "https";
  tool?: "python" | "git" | "node" | "java";
  version?: string;
  extension?: string;
  min_free_gb?: number;
  url?: string;
  required: boolean;
}
export interface Profile {
  schema_version: number;
  id: string;
  name: string;
  checks: Check[];
}
export interface CheckResult {
  check_id: string;
  status: Status;
  required: boolean;
  installed_version: string | null;
  reason_code: string;
  message_sl: string;
  suggestion_sl: string;
}
export interface Report {
  schema_version: number;
  results: CheckResult[];
  summary: {
    total: number;
    pass: number;
    fail: number;
    warning: number;
    unknown: number;
    ready: boolean;
  };
}
export const statusLabels: Record<Status, string> = {
  pass: "V redu",
  fail: "Potrebna ureditev",
  warning: "Opozorilo",
  unknown: "Ni mogoče potrditi",
};
export const toolNames = {
  python: "Python",
  git: "Git",
  node: "Node.js",
  java: "Java / JDK",
};

export function checkName(check: Check): string {
  return check.type === "runtime" && check.tool
    ? toolNames[check.tool]
    : check.type === "vscode_extension"
      ? "VS Code razširitev"
      : check.type === "disk_space"
        ? "Prostor na disku"
        : "HTTPS povezava";
}
export function checkRequirement(check: Check): string {
  return check.type === "runtime"
    ? (check.version ?? "")
    : check.type === "vscode_extension"
      ? (check.extension ?? "")
      : check.type === "disk_space"
        ? "vsaj " + check.min_free_gb + " GB"
        : (check.url ?? "");
}
export function checkIcon(check: Check): string {
  return check.type === "runtime"
    ? { python: "Py", git: "Gt", node: "JS", java: "Jv" }[
        check.tool ?? "python"
      ]
    : check.type === "vscode_extension"
      ? "VS"
      : check.type === "disk_space"
        ? "GB"
        : "↗";
}
