import { expect, test, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import ProfileEditor from "./ProfileEditor";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
test("validates before applying and displays safe validation errors", async () => {
  const user = userEvent.setup();
  const apply = vi.fn();
  vi.mocked(invoke).mockRejectedValueOnce({
    message_sl: "Profil ni veljaven.",
  });
  render(<ProfileEditor disabled={false} onApply={apply} />);
  await user.click(screen.getByRole("button", { name: "Preveri in uporabi" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Profil ni veljaven.",
  );
  expect(apply).not.toHaveBeenCalled();
  vi.mocked(invoke).mockResolvedValueOnce({ id: "moja-delavnica", checks: [] });
  await user.click(screen.getByRole("button", { name: "Preveri in uporabi" }));
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({ id: "moja-delavnica" }),
    expect.stringContaining("schema_version: 1"),
  );
});
test("validates before opening export dialog and reports cancellation", async () => {
  const user = userEvent.setup();
  vi.mocked(invoke)
    .mockReset()
    .mockResolvedValueOnce({ id: "moja-delavnica" })
    .mockResolvedValueOnce(false);
  render(<ProfileEditor disabled={false} onApply={vi.fn()} />);
  await user.click(screen.getByRole("button", { name: "Izvozi YAML" }));
  expect(await screen.findByRole("status")).toHaveTextContent("preklicano");
  expect(invoke).toHaveBeenLastCalledWith(
    "export_profile",
    expect.objectContaining({ profileId: "moja-delavnica" }),
  );
});
