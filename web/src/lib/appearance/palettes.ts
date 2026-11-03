export const palettes = {
  mineral: { label: "Mineral", accent: "#087f78", canvas: "#f4f6f8", sidebar: "#17232d" },
  ocean: { label: "Ocean", accent: "#2463ab", canvas: "#f2f6fa", sidebar: "#172638" },
  iris: { label: "Iris", accent: "#7251a1", canvas: "#f6f4f9", sidebar: "#272137" },
  rose: { label: "Rose", accent: "#a33d60", canvas: "#faf5f6", sidebar: "#30212b" },
} as const;

export type Palette = keyof typeof palettes;
export type ThemeMode = "light" | "dark" | "system";
export interface Appearance {
  mode: ThemeMode;
  palette: Palette;
  accent: string;
  background: string;
  sidebar: string;
  radius: number;
}
export const defaultAppearance: Appearance = {
  mode: "system", palette: "mineral", accent: "", background: "", sidebar: "", radius: 8,
};
export const normalizeAppearance = (value: unknown): Appearance => {
  const input = value && typeof value === "object" ? value as Partial<Appearance> : {};
  const color = (v: unknown) => typeof v === "string" && /^#[0-9a-f]{6}$/i.test(v) ? v : "";
  return {
    mode: ["light", "dark", "system"].includes(String(input.mode)) ? input.mode as ThemeMode : "system",
    palette: input.palette && Object.prototype.hasOwnProperty.call(palettes, input.palette) ? input.palette : "mineral",
    accent: color(input.accent), background: color(input.background), sidebar: color(input.sidebar),
    radius: [4, 8, 12].includes(Number(input.radius)) ? Number(input.radius) : 8,
  };
};
