import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import "@testing-library/jest-dom/vitest";
import { AppearanceSettings } from "./appearance-settings";

const mocks = vi.hoisted(() => ({
  setTheme: vi.fn(),
  updateAppearance: vi.fn(),
  resetAppearance: vi.fn(),
}));

vi.mock("@/lib/i18n", () => ({ useI18n: () => ({ t: (value: string) => value }) }));
vi.mock("@/lib/theme", () => ({
  useTheme: () => ({
    appearance: { mode: "system", palette: "mineral", accent: "", background: "", sidebar: "", radius: 8 },
    setTheme: mocks.setTheme,
    updateAppearance: mocks.updateAppearance,
    resetAppearance: mocks.resetAppearance,
  }),
}));

describe("appearance settings", () => {
  afterEach(cleanup);
  beforeEach(() => vi.clearAllMocks());

  it("applies mode, palette, custom color, and radius choices immediately", async () => {
    const user = userEvent.setup();
    render(<AppearanceSettings />);
    await user.click(screen.getByRole("button", { name: "Palette settings" }));

    await user.click(screen.getByRole("button", { name: "Dark" }));
    await user.click(screen.getByRole("button", { name: "Ocean" }));
    fireEvent.change(screen.getByLabelText("Accent color"), { target: { value: "#112233" } });
    await user.click(screen.getByRole("button", { name: "12px" }));

    expect(mocks.setTheme).toHaveBeenCalledWith("dark");
    expect(mocks.updateAppearance).toHaveBeenCalledWith({ palette: "ocean" });
    expect(mocks.updateAppearance).toHaveBeenCalledWith({ accent: "#112233" });
    expect(mocks.updateAppearance).toHaveBeenCalledWith({ radius: 12 });
    expect(screen.getByText("Preferences are saved automatically in this browser.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Close" }));
  });

  it("resets custom appearance preferences when requested", async () => {
    const user = userEvent.setup();
    render(<AppearanceSettings compact />);
    await user.click(screen.getByRole("button", { name: "Palette settings" }));
    await user.click(screen.getByRole("button", { name: "Reset appearance" }));

    expect(mocks.resetAppearance).toHaveBeenCalledTimes(1);
    await user.click(screen.getByRole("button", { name: "Close" }));
  });
});
