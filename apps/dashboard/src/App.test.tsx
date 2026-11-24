import { beforeEach, expect, test, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { api, ApiError } from "./api";
vi.mock("./api", async () => ({
  ...(await vi.importActual("./api")),
  api: vi.fn(),
}));
const room = {
  room_code: "ABCDEFGH",
  teacher_token: "a".repeat(64),
  expires_at: Math.floor(Date.now() / 1000) + 1000,
};
const data = {
  sessions: [
    { session_id: "1", alias: "Udeleženec-1", results: null },
    {
      session_id: "2",
      alias: "Udeleženec-2",
      results: [
        { check_id: "python", status: "fail", installed_version: null },
      ],
    },
  ],
  aggregate: { pass: 0, fail: 1, warning: 0, unknown: 0 },
  expires_at: room.expires_at,
};
beforeEach(() => {
  sessionStorage.clear();
  vi.mocked(api).mockReset();
});
test("creates a room, preserves secret in session only, displays two participants", async () => {
  vi.mocked(api).mockResolvedValueOnce(room).mockResolvedValue(data);
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Ustvari sobo" }));
  expect(await screen.findByText("Udeleženec-2")).toBeInTheDocument();
  expect(screen.getByText("Rezultati še niso deljeni.")).toBeInTheDocument();
  expect(sessionStorage.getItem("preflight-room")).toContain(
    room.teacher_token,
  );
  expect(location.href).not.toContain(room.teacher_token);
  expect(screen.getAllByLabelText("Učiteljska skrivnost")[0]).toHaveAttribute(
    "type",
    "password",
  );
});
test("restores existing room and stops on expiry", async () => {
  sessionStorage.setItem("preflight-room", JSON.stringify(room));
  vi.mocked(api).mockRejectedValue(new ApiError(404));
  render(<App />);
  expect(await screen.findByRole("alert")).toHaveTextContent("potekla");
  await waitFor(() =>
    expect(screen.getByText(/Osveževanje ustavljeno/)).toBeInTheDocument(),
  );
  expect(api).toHaveBeenCalledTimes(1);
});
test("filter shows only problematic checks and deletion clears saved access", async () => {
  sessionStorage.setItem("preflight-room", JSON.stringify(room));
  vi.mocked(api).mockResolvedValue(data);
  const user = userEvent.setup();
  render(<App />);
  await screen.findByText("Udeleženec-2");
  await user.selectOptions(screen.getByLabelText("Težavna preverba"), "python");
  expect(screen.queryByText("Udeleženec-1")).not.toBeInTheDocument();
  await user.click(screen.getByText("Zaključi delavnico"));
  await user.click(
    screen.getByRole("button", { name: "Izbriši sobo in rezultate" }),
  );
  expect(
    await screen.findByRole("button", { name: "Ustvari sobo" }),
  ).toBeInTheDocument();
  expect(sessionStorage.getItem("preflight-room")).toBeNull();
});
