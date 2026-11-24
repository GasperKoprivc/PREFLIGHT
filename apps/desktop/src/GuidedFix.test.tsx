import { expect, test, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import GuidedFix from "./GuidedFix";
test("copies a static command, exposes official links and rechecks without installation", async () => {
  const user = userEvent.setup();
  const recheck = vi.fn();
  render(
    <GuidedFix
      check={{
        id: "git",
        type: "runtime",
        tool: "git",
        required: true,
        version: ">=2.40.0",
      }}
      result={{
        check_id: "git",
        status: "fail",
        required: true,
        installed_version: null,
        reason_code: "not_found",
        message_sl: "",
        suggestion_sl: "",
      }}
      platform="windows"
      disabled={false}
      onRecheck={recheck}
    />,
  );
  expect(
    screen.getByRole("link", { name: /Uradna namestitev/ }),
  ).toHaveAttribute("href", "https://git-scm.com/install/");
  await user.click(screen.getAllByRole("button", { name: "Kopiraj ukaz" })[0]);
  expect(await screen.findByRole("status")).toHaveTextContent("kopiran");
  expect(await navigator.clipboard.readText()).toBe("git --version");
  await user.click(
    screen.getByRole("button", { name: "Ponovno preveri to zahtevo" }),
  );
  expect(recheck).toHaveBeenCalledOnce();
});
