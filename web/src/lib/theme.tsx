import { createContext, useContext, useLayoutEffect, useMemo, useSyncExternalStore } from "react";
import { appearanceColors } from "./appearance/colors";
import { useAppearanceStore } from "./appearance/store";
import type { Appearance, ThemeMode } from "./appearance/palettes";

interface ThemeContextValue {
  theme: "light" | "dark";
  editorTheme: "gity-light" | "gity-dark";
  appearance: Appearance;
  setTheme: (theme: ThemeMode) => void;
  toggleTheme: () => void;
  updateAppearance: (patch: Partial<Appearance>) => void;
  resetAppearance: () => void;
}
const ThemeContext = createContext<ThemeContextValue | null>(null);
const query = "(prefers-color-scheme: dark)";
const subscribeSystem = (callback: () => void) => {
  const media = window.matchMedia(query);
  media.addEventListener("change", callback);
  return () => media.removeEventListener("change", callback);
};
export const ThemeProvider = ({ children }: { children: React.ReactNode }): JSX.Element => {
  const appearance = useAppearanceStore((state) => state.appearance);
  const update = useAppearanceStore((state) => state.update);
  const reset = useAppearanceStore((state) => state.reset);
  const systemDark = useSyncExternalStore(subscribeSystem, () => window.matchMedia(query).matches, () => false);
  const dark = appearance.mode === "dark" || (appearance.mode === "system" && systemDark);
  const theme = dark ? "dark" : "light";
  useLayoutEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", dark);
    root.dataset.theme = theme;
    root.dataset.palette = appearance.palette;
    root.style.colorScheme = theme;
    for (const [key, value] of Object.entries(appearanceColors(appearance, dark))) root.style.setProperty(key, value);
    root.style.setProperty("--radius", `${appearance.radius}px`);
  }, [appearance, dark, theme]);
  useLayoutEffect(() => {
    const listener = (event: StorageEvent) => {
      if (event.key === "gity.appearance") void useAppearanceStore.persist.rehydrate();
    };
    window.addEventListener("storage", listener);
    return () => window.removeEventListener("storage", listener);
  }, []);
  const value = useMemo<ThemeContextValue>(() => ({
    theme, appearance, editorTheme: `gity-${theme}`,
    setTheme: (mode) => update({ mode }), toggleTheme: () => update({ mode: dark ? "light" : "dark" }),
    updateAppearance: update, resetAppearance: reset,
  }), [theme, appearance, update, reset, dark]);
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};

export const useTheme = (): ThemeContextValue => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within ThemeProvider");
  }
  return context;
};
