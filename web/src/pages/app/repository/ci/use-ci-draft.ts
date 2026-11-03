import { useEffect, useState } from "react";
import { z } from "zod";
const draftSchema = z.object({
  source: z.string().max(1024 * 1024),
  original: z.string(),
  head: z.string(),
  exists: z.boolean(),
});
export type CIDraft = z.infer<typeof draftSchema>;
/** Per-tab recovery only; commit always uses the original expected Git revision. */
export const useCIDraft = (key: string, loaded: boolean, current: CIDraft) => {
  const [checked, setChecked] = useState<string | null>(null);
  const [recoverable, setRecoverable] = useState<CIDraft | null>(null);
  useEffect(() => {
    if (!loaded || checked === key) return;
    try {
      const text = sessionStorage.getItem(key);
      const parsed = text ? draftSchema.safeParse(JSON.parse(text)) : null;
      if (parsed?.success && parsed.data.source !== current.source && parsed.data.source !== parsed.data.original) {
        setRecoverable(parsed.data);
        return;
      }
    } catch {
      /* Storage can be unavailable under browser privacy policies. */
    }
    setRecoverable(null);
    setChecked(key);
  }, [key, loaded, checked, current.source]);
  useEffect(() => {
    if (!loaded || checked !== key || recoverable) return;
    try {
      if (current.source === current.original) sessionStorage.removeItem(key);
      else sessionStorage.setItem(key, JSON.stringify(current));
    } catch {
      /* The native unload warning still protects the in-memory draft. */
    }
  }, [key, loaded, checked, recoverable, current.source, current.original, current.head, current.exists]);
  const discard = () => {
    try {
      sessionStorage.removeItem(key);
    } catch {
      /* Optional recovery storage. */
    }
  };
  const resolve = () => {
    setRecoverable(null);
    setChecked(key);
  };
  return { recoverable, discard, resolve };
};
