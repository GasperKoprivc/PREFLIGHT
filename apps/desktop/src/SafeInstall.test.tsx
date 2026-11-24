import { beforeEach, expect, test, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import SafeInstall from "./SafeInstall";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const report = {
  schema_version: 1,
  results: [],
  summary: { total: 0, pass: 0, fail: 0, warning: 0, unknown: 0, ready: true },
};
const prepared = {
  nonce: "nonce",
  action: {
    name_sl: "Git za Windows",
    manager: "winget",
    package_id: "Git.Git",
    version: "2.56.1",
    command: "winget install --id Git.Git",
    effects_sl: "Namesti Git.",
    permissions_sl: "Lahko zahteva skrbnika.",
    license_url: "https://git-scm.com/about/free-and-open-source",
  },
};
beforeEach(() => {
  vi.mocked(invoke).mockReset();
});
test("preview and cancellation never install; separate confirmation invokes only nonce", async () => {
  const user = userEvent.setup();
  vi.mocked(invoke).mockResolvedValue(prepared);
  const onReport = vi.fn();
  render(
    <SafeInstall
      profileId="p"
      checkId="git"
      report={report}
      disabled={false}
      onBusy={vi.fn()}
      onReport={onReport}
    />,
  );
  expect(invoke).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole("button", { name: "Preglej namestitev Git" }),
  );
  expect(
    await screen.findByRole("region", { name: "Potrditev namestitve" }),
  ).toHaveTextContent("Git.Git");
  await user.click(screen.getByRole("button", { name: "Prekliči" }));
  expect(invoke).toHaveBeenCalledTimes(1);
  await user.click(
    screen.getByRole("button", { name: "Preglej namestitev Git" }),
  );
  await screen.findByRole("button", { name: "Potrdi namestitev" });
  vi.mocked(invoke).mockResolvedValue({
    message_sl: "Končano.",
    report,
    uncertain: false,
  });
  await user.click(screen.getByRole("button", { name: "Potrdi namestitev" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("confirm_fix", {
      nonce: "nonce",
      confirmed: true,
    }),
  );
  expect(onReport).toHaveBeenCalledWith(report);
});
test("unavailable manager is explained without fake success", async () => {
  const user = userEvent.setup();
  vi.mocked(invoke).mockRejectedValue({ message_sl: "Winget ni na voljo." });
  render(
    <SafeInstall
      profileId="p"
      checkId="git"
      report={report}
      disabled={false}
      onBusy={vi.fn()}
      onReport={vi.fn()}
    />,
  );
  await user.click(
    screen.getByRole("button", { name: "Preglej namestitev Git" }),
  );
  expect(await screen.findByRole("status")).toHaveTextContent(
    "Winget ni na voljo.",
  );
  expect(
    screen.queryByRole("button", { name: "Potrdi namestitev" }),
  ).not.toBeInTheDocument();
});
