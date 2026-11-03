import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import { defaultAppearance, normalizeAppearance, type Appearance } from "./palettes";

interface AppearanceState {
  appearance: Appearance;
  update: (patch: Partial<Appearance>) => void;
  reset: () => void;
}
export const useAppearanceStore = create<AppearanceState>()(persist((set) => ({
  appearance: defaultAppearance,
  update: (patch) => set((state) => ({ appearance: normalizeAppearance({ ...state.appearance, ...patch }) })),
  reset: () => set({ appearance: { ...defaultAppearance } }),
}), {
  name: "gity.appearance", version: 1,
  storage: createJSONStorage(() => localStorage),
  partialize: (state) => ({ appearance: state.appearance }),
  merge: (stored, current) => ({ ...current, appearance: normalizeAppearance((stored as Partial<AppearanceState>)?.appearance) }),
}));
