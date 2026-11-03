import { createContext, useContext, useEffect, useMemo, useState } from "react";
import { translateCatalogText } from "./i18n-catalog";

export type Locale = "en" | "zh";

interface I18nContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  toggleLocale: () => void;
  t: (text: string) => string;
}

const STORAGE_KEY = "gity.locale";
const I18nContext = createContext<I18nContextValue | null>(null);

export const resolveStoredLocale = (): Locale => {
  if (typeof window === "undefined") return "en";
  const stored = window.localStorage.getItem(STORAGE_KEY);
  return stored === "en" || stored === "zh" ? stored : "en";
};

export const translateText = translateCatalogText;

export const translateForCurrentLocale = (text: string): string => translateText(text, resolveStoredLocale());

export const I18nProvider = ({ children }: { children: React.ReactNode }): JSX.Element => {
  const [locale, setLocaleState] = useState<Locale>(() => resolveStoredLocale());

  useEffect(() => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(STORAGE_KEY, locale);
    document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
  }, [locale]);

  const value = useMemo<I18nContextValue>(
    () => ({
      locale,
      setLocale: setLocaleState,
      toggleLocale: () => setLocaleState((current) => current === "en" ? "zh" : "en"),
      t: (text) => translateText(text, locale),
    }),
    [locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
};

export const useI18n = (): I18nContextValue => {
  const context = useContext(I18nContext);
  if (!context) throw new Error("useI18n must be used within I18nProvider");
  return context;
};
