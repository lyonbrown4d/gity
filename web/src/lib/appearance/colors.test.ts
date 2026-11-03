import { describe, it, expect } from "vitest";
import { colord } from "colord";
import { appearanceColors, readableForeground } from "./colors";
import { defaultAppearance, normalizeAppearance, palettes } from "./palettes";

describe("shared appearance", () => {
  it("chooses readable button foregrounds for arbitrary custom accents", () => {
    for (const color of ["#ffffff", "#000000", "#aabbcc", "#00ff00", "#ff00ff", "#087f78"]) {
      expect(colord(color).contrast(readableForeground(color))).toBeGreaterThanOrEqual(4.5);
    }
  });
  it("provides paired semantic tokens for each preset and both modes", () => {
    for (const palette of Object.keys(palettes) as Array<keyof typeof palettes>) {
      for (const dark of [false, true]) {
        const tokens = appearanceColors({ ...defaultAppearance, palette }, dark);
        const color = (key: string) => `hsl(${tokens[`--${key}`]})`;
        expect(colord(color("background")).contrast(color("foreground"))).toBeGreaterThanOrEqual(7);
        expect(colord(color("card")).contrast(color("muted-foreground"))).toBeGreaterThanOrEqual(4.5);
        expect(colord(color("sidebar-background")).contrast(color("sidebar-foreground"))).toBeGreaterThanOrEqual(4.5);
      }
    }
  });
  it("rejects invalid or injectable persisted settings", () => {
    expect(normalizeAppearance({ accent: "red; color: red", radius: 900, palette: "__proto__" }))
      .toEqual(defaultAppearance);
  });
});
