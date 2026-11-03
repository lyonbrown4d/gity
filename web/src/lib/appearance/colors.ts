import { colord, extend } from "colord";
import a11yPlugin from "colord/plugins/a11y";
import mixPlugin from "colord/plugins/mix";
import { palettes, type Appearance } from "./palettes";

extend([a11yPlugin, mixPlugin]);
export const readableForeground = (background: string): string => {
  const c = colord(background);
  return c.contrast("#ffffff") >= c.contrast("#000000") ? "#ffffff" : "#000000";
};
const contrasted = (color: string, background: string, minimum = 4.5): string => {
  const target = readableForeground(background);
  for (let step = 0; step <= 20; step += 1) {
    const candidate = colord(color).mix(target, step / 20);
    if (candidate.contrast(background) >= minimum) return candidate.toHex();
  }
  return target;
};
const hsl = (color: string): string => {
  const { h, s, l } = colord(color).toHsl();
  return `${h} ${s}% ${l}%`;
};
export const appearanceColors = (appearance: Appearance, dark: boolean): Record<string, string> => {
  const preset = palettes[appearance.palette];
  const background = appearance.background || (dark ? "#141b23" : preset.canvas);
  const isDark = colord(background).isDark();
  const foreground = isDark ? "#edf2f7" : "#182530";
  const safeForeground = colord(background).contrast(foreground) >= 7 ? foreground : readableForeground(background);
  const surface = colord(background).mix("#ffffff", isDark ? 0.04 : 0.85).toHex();
  const muted = colord(background).mix(safeForeground, 0.06).toHex();
  const mutedForeground = colord(safeForeground).mix(background, 0.25).toHex();
  const border = colord(background).mix(safeForeground, 0.15).toHex();
  const primary = contrasted(appearance.accent || preset.accent, background);
  const sidebar = appearance.sidebar || (dark ? "#111923" : preset.sidebar);
  const sidebarForeground = readableForeground(sidebar);
  const sidebarPrimary = contrasted(primary, sidebar, 4.5);
  const values: Record<string, string> = {
    background, foreground: safeForeground, card: surface, "card-foreground": safeForeground,
    popover: surface, "popover-foreground": safeForeground, primary, "primary-foreground": readableForeground(primary),
    secondary: muted, "secondary-foreground": safeForeground, muted, "muted-foreground": mutedForeground,
    accent: colord(background).mix(primary, 0.1).toHex(), "accent-foreground": safeForeground,
    border, input: border, ring: primary, "sidebar-background": sidebar,
    "sidebar-foreground": sidebarForeground, "sidebar-primary": sidebarPrimary,
    "sidebar-primary-foreground": readableForeground(sidebarPrimary),
    "sidebar-accent": colord(sidebar).mix(sidebarForeground, 0.1).toHex(),
    "sidebar-accent-foreground": sidebarForeground,
    "sidebar-border": colord(sidebar).mix(sidebarForeground, 0.15).toHex(), "sidebar-ring": sidebarPrimary,
    "canvas-subtle": muted, "canvas-strong": border,
    success: contrasted(isDark ? "#65d9a0" : "#19744b", background),
    warning: contrasted(isDark ? "#f0be58" : "#946000", background),
    destructive: contrasted(isDark ? "#ff8a94" : "#ba3f46", background),
    "destructive-foreground": readableForeground(contrasted(isDark ? "#ff8a94" : "#ba3f46", background)),
    "chart-1": primary, "chart-2": "#2463ab", "chart-3": "#946000", "chart-4": "#ba3f46", "chart-5": "#7251a1",
  };
  return Object.fromEntries(Object.entries(values).map(([key, value]) => [`--${key}`, hsl(value)]));
};
