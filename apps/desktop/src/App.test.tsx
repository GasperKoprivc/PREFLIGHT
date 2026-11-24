import { beforeEach, expect, test, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import type { Profile, Report } from "./model";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const invokeMock = vi.mocked(invoke);
const profile: Profile = {
  schema_version: 1,
  id: "python-workshop",
  name: "Python delavnica",
  checks: [
    {
      id: "python",
      type: "runtime",
      tool: "python",
      version: ">=3.11.0",
      required: true,
    },
  ],
};
const report: Report = {
  schema_version: 1,
  results: [
    {
      check_id: "python",
      status: "pass",
      required: true,
      installed_version: "3.12.0",
      reason_code: "satisfied",
      message_sl: "Zahtevana različica je na voljo.",
      suggestion_sl: "Okolje je pripravljeno.",
    },
  ],
  summary: { total: 1, pass: 1, fail: 0, warning: 0, unknown: 0, ready: true },
};
beforeEach(() => {
  localStorage.clear();
  invokeMock.mockReset();
  invokeMock.mockImplementation(async (command) => {
    if (command === "list_profiles") return [profile];
    if (command === "get_app_info")
      return { name: "Preflight", version: "0.1.0" };
    if (command === "run_checks") return report;
    throw new Error("Unexpected IPC");
  });
});

test("shows unverified state then genuine IPC result and supports rerun", async () => {
  const user = userEvent.setup();
  render(<App />);
  const run = await screen.findByRole("button", { name: "Zaženi preverjanje" });
  await waitFor(() => expect(run).toBeEnabled());
  expect(screen.getByText("Še ni preverjeno")).toBeInTheDocument();
  expect(screen.queryByText("3.12.0")).not.toBeInTheDocument();
  await user.click(run);
  expect(
    await screen.findByText("Obvezne zahteve so izpolnjene."),
  ).toHaveFocus();
  expect(screen.getByText("3.12.0")).toBeInTheDocument();
  expect(invokeMock).toHaveBeenCalledWith("run_checks", {
    profileId: "python-workshop",
    profileYaml: null,
    allowNetwork: false,
  });
  await user.click(screen.getByRole("button", { name: "Ponovno preveri" }));
  await waitFor(() =>
    expect(
      invokeMock.mock.calls.filter(([c]) => c === "run_checks"),
    ).toHaveLength(2),
  );
});

test("prevents duplicate runs and profile changes while pending", async () => {
  const user = userEvent.setup();
  let finish!: (value: Report) => void;
  const pending = new Promise<Report>((resolve) => {
    finish = resolve;
  });
  invokeMock.mockImplementation(async (command) =>
    command === "run_checks"
      ? pending
      : command === "list_profiles"
        ? [profile]
        : { version: "0.1.0" },
  );
  render(<App />);
  const run = await screen.findByRole("button", { name: "Zaženi preverjanje" });
  await waitFor(() => expect(run).toBeEnabled());
  await user.click(run);
  expect(
    screen.getByRole("button", { name: "Preverjanje poteka …" }),
  ).toBeDisabled();
  expect(
    screen.getByRole("combobox", { name: "Izberi delavnico" }),
  ).toBeDisabled();
  expect(screen.getByText("Preverjamo tvoje okolje …")).toBeInTheDocument();
  finish(report);
  expect(
    await screen.findByRole("button", { name: "Ponovno preveri" }),
  ).toBeEnabled();
});

test("unknown timeout is not shown as ready and details explain it", async () => {
  const user = userEvent.setup();
  const unknown: Report = {
    ...report,
    results: [
      {
        ...report.results[0],
        status: "unknown",
        installed_version: null,
        reason_code: "timeout",
        message_sl: "Program se ni odzval v časovni omejitvi.",
        suggestion_sl: "Poskusi znova.",
      },
    ],
    summary: {
      total: 1,
      pass: 0,
      fail: 0,
      warning: 0,
      unknown: 1,
      ready: false,
    },
  };
  invokeMock.mockImplementation(async (command) =>
    command === "run_checks"
      ? unknown
      : command === "list_profiles"
        ? [profile]
        : { version: "0.1.0" },
  );
  render(<App />);
  const run = await screen.findByRole("button", { name: "Zaženi preverjanje" });
  await waitFor(() => expect(run).toBeEnabled());
  await user.click(run);
  expect(
    await screen.findByText("Nekaj stvari še potrebuje pozornost."),
  ).toBeInTheDocument();
  expect(screen.getByText("Ni mogoče potrditi")).toBeInTheDocument();
  await user.click(screen.getByText("Python"));
  expect(
    screen.getByText(
      "Časovna omejitev ne pomeni, da program manjka. Poskusi s ponovnim preverjanjem.",
    ),
  ).toBeVisible();
});

test("shows safe errors and can retry after failure", async () => {
  const user = userEvent.setup();
  let attempts = 0;
  invokeMock.mockImplementation(async (command) => {
    if (command === "list_profiles") return [profile];
    if (command === "run_checks") {
      if (attempts++ === 0) throw new Error("C:/Users/private");
      return report;
    }
    return { version: "0.1.0" };
  });
  render(<App />);
  const run = await screen.findByRole("button", { name: "Zaženi preverjanje" });
  await waitFor(() => expect(run).toBeEnabled());
  await user.click(run);
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Lokalno jedro ni dosegljivo",
  );
  expect(screen.queryByText(/C:\/Users/)).not.toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Zaženi preverjanje" }));
  expect(
    await screen.findByText("Obvezne zahteve so izpolnjene."),
  ).toBeInTheDocument();
});

test("empty selection clears old results and disables execution", async () => {
  const user = userEvent.setup();
  render(<App />);
  await screen.findByRole("option", { name: "Python delavnica" });
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Izberi delavnico" }),
    "",
  );
  expect(screen.getByText("Začni z izbiro profila.")).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "Zaženi preverjanje" }),
  ).toBeDisabled();
});

test("theme and privacy pages work without network commands", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Nastavitve" }));
  await user.selectOptions(screen.getByLabelText("Barvna tema"), "light");
  expect(document.documentElement.dataset.theme).toBe("light");
  expect(localStorage.getItem("preflight-theme")).toBe("light");
  await user.click(screen.getByRole("button", { name: "Zasebnost" }));
  expect(screen.getByText("Deljenje je izključeno.")).toBeInTheDocument();
  expect(
    invokeMock.mock.calls.every(([c]) =>
      ["list_profiles", "get_app_info"].includes(c),
    ),
  ).toBe(true);
});

test("startup failure offers retry and empty list is explained", async () => {
  const user = userEvent.setup();
  let attempts = 0;
  invokeMock.mockImplementation(async (command) => {
    if (command === "list_profiles") {
      if (attempts++ === 0) throw new Error("offline");
      return [];
    }
    return { version: "0.1.0" };
  });
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Poskusi znova" }),
  );
  expect(
    await screen.findByText("Trenutno ni na voljo noben profil."),
  ).toBeInTheDocument();
});

test("individual recheck sends only the selected check ID", async () => {
  const user = userEvent.setup();
  render(<App />);
  const start = await screen.findByRole("button", {
    name: "Zaženi preverjanje",
  });
  await waitFor(() => expect(start).toBeEnabled());
  await user.click(start);
  await screen.findByText("Obvezne zahteve so izpolnjene.");
  await user.click(screen.getByText("Python"));
  await user.click(
    screen.getByRole("button", { name: "Ponovno preveri to zahtevo" }),
  );
  await waitFor(() =>
    expect(invokeMock).toHaveBeenLastCalledWith("run_checks", {
      profileId: "python-workshop",
      profileYaml: null,
      allowNetwork: false,
      checkId: "python",
    }),
  );
});
