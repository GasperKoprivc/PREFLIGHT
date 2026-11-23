export type Status = "pass" | "fail" | "warning" | "unknown";
export interface Check {
  id: string;
  type: "runtime";
  tool: "python" | "git" | "node" | "java";
  version: string;
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
