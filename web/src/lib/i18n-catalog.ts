export type Locale = "en" | "zh";

export interface LocaleEntry {
  en: string;
  zh: string;
}

type CatalogChunk = Record<string, LocaleEntry | number>;

const sharedLocaleChunks = import.meta.glob<CatalogChunk>(
  "../../../crates/gity-i18n/locales/*.json",
  { eager: true, import: "default" },
);

export const I18N_CATALOG: Record<string, LocaleEntry> = Object.values(sharedLocaleChunks)
  .reduce<Record<string, LocaleEntry>>((catalog, chunk) => {
    if (chunk._version !== 2) return catalog;
    for (const [key, value] of Object.entries(chunk)) {
      if (key !== "_version" && typeof value !== "number") catalog[key] = value;
    }
    return catalog;
  }, {});

export const normalizePlaceholders = (text: string): string => text.replace(/%\{([A-Za-z_][A-Za-z_0-9]*)\}/g, "{$1}");

export const translateCatalogText = (text: string, locale: Locale): string => {
  const entry = I18N_CATALOG[text];
  return normalizePlaceholders(entry?.[locale] ?? text);
};
