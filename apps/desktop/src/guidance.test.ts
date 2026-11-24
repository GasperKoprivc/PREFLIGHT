import { expect, test } from "vitest";
import { guidance, type OperatingSystem } from "./guidance";
import type { Check, CheckResult } from "./model";
const check: Check = {
  id: "x",
  type: "runtime",
  tool: "python",
  version: ">=3.11.0",
  required: true,
};
function result(reason_code: string): CheckResult {
  return {
    check_id: "x",
    status: "fail",
    required: true,
    installed_version: null,
    reason_code,
    message_sl: "",
    suggestion_sl: "",
  };
}
test("distinguishes missing Python, version mismatch and JDK", () => {
  expect(
    guidance(check, result("not_found"), "windows").steps.join(" "),
  ).toContain("py -3");
  expect(guidance(check, result("version_mismatch"), "macos").title).toContain(
    "različico",
  );
  expect(
    guidance({ ...check, tool: "java" }, result("jdk_missing"), "linux").title,
  ).toContain("JDK");
  expect(
    guidance(
      { ...check, tool: "java" },
      result("jdk_mismatch"),
      "windows",
    ).steps.join(" "),
  ).toContain("JAVA_HOME");
});
test("unknown results and unsupported OS never claim missing software", () => {
  expect(guidance(check, result("timeout"), "windows").title).toContain(
    "potrditi",
  );
  expect(guidance(check, result("not_found"), "other").commands).toEqual([]);
});
test("extension commands are restricted to trusted catalog", () => {
  const ext: Check = {
    id: "ext",
    type: "vscode_extension",
    extension: "ms-python.python",
    required: true,
  };
  expect(
    guidance(ext, result("extension_missing"), "windows").commands.some(
      (c) => c.command === "code --install-extension ms-python.python",
    ),
  ).toBe(true);
  for (const extension of [
    "publisher.custom",
    "ms-python.python; whoami",
    "$(secret)",
  ]) {
    const commands = guidance(
      { ...ext, extension },
      result("extension_missing"),
      "linux",
    ).commands;
    expect(commands.some((c) => c.command.includes(extension))).toBe(false);
  }
});
test("all platforms and reasons yield static official links and safe commands", () => {
  const reasons = [
    "not_found",
    "version_mismatch",
    "jdk_missing",
    "jdk_mismatch",
    "timeout",
    "output_limit",
    "process_failed",
    "unrecognized_version",
  ];
  for (const os of ["windows", "macos", "linux"] as OperatingSystem[])
    for (const tool of ["python", "git", "node", "java"] as const)
      for (const reason of reasons) {
        const guide = guidance({ ...check, tool }, result(reason), os);
        expect(guide.steps.length).toBeGreaterThan(0);
        for (const link of guide.links)
          expect(new URL(link.url).protocol).toBe("https:");
        for (const command of guide.commands)
          expect(command.command).not.toMatch(/[;&|`$\n\r]/);
      }
});
