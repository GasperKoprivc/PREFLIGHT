import { beforeEach, expect, test, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import Classroom from "./Classroom";
import type { Report } from "./model";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const profile = { schema_version: 1, id: "x", name: "Delavnica", checks: [] };
const room = {
  code: "ABCDEFGH",
  profile,
  yaml: "schema_version: 1",
  expires_at: 9999999999,
  shared: false,
};
const report: Report = {
  schema_version: 1,
  results: [],
  summary: { total: 0, pass: 0, fail: 0, warning: 0, unknown: 0, ready: true },
};
beforeEach(() => {
  vi.mocked(invoke).mockReset();
});
test("fetches profile without sharing, requires opt-in and supports deletion", async () => {
  vi.mocked(invoke).mockResolvedValue(room);
  const user = userEvent.setup();
  const apply = vi.fn();
  render(<Classroom running={false} report={report} onApply={apply} />);
  await user.type(screen.getByLabelText("Koda sobe"), "ABCDEFGH");
  await user.click(screen.getByRole("button", { name: "Prenesi profil sobe" }));
  expect(await screen.findByText("Soba ABCDEFGH")).toBeInTheDocument();
  expect(apply).toHaveBeenCalled();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("button", { name: "Deli rezultate" })).toBeDisabled();
  await user.click(screen.getByRole("checkbox"));
  await user.click(screen.getByRole("button", { name: "Deli rezultate" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("room_share", { consent: true }),
  );
  await user.click(
    screen.getByRole("button", { name: "Izklopi deljenje in izbriši sejo" }),
  );
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("room_leave"));
});
test("failed removal keeps controls for retry", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce(room)
    .mockRejectedValue({ message_sl: "Povezava ni uspela." });
  const user = userEvent.setup();
  render(<Classroom running={false} report={null} onApply={vi.fn()} />);
  await user.type(screen.getByLabelText("Koda sobe"), "ABCDEFGH");
  await user.click(screen.getByRole("button", { name: "Prenesi profil sobe" }));
  await screen.findByText("Soba ABCDEFGH");
  await user.click(
    screen.getByRole("button", { name: "Izklopi deljenje in izbriši sejo" }),
  );
  expect(await screen.findByRole("status")).toHaveTextContent(
    "Povezava ni uspela.",
  );
  expect(screen.getByText("Soba ABCDEFGH")).toBeInTheDocument();
});
