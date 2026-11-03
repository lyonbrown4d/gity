// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import "@testing-library/jest-dom/vitest";
import { I18N_CATALOG, translateCatalogText } from "./i18n-catalog";
import { I18nProvider, useI18n } from "./i18n";

const LocaleHarness = (): JSX.Element => {
  const { locale, toggleLocale, t } = useI18n();
  return (
    <div>
      <span>{locale}</span>
      <span>{t("Welcome back")}</span>
      <button type="button" onClick={toggleLocale}>Toggle</button>
    </div>
  );
};

describe("embedded UI translations", () => {
  it("loads complete bilingual entries with matching placeholders", () => {
    const placeholderNames = (text: string) => [...text.matchAll(/%\{([A-Za-z_][A-Za-z_0-9]*)\}/g)]
      .map((match) => match[1])
      .sort();
    expect(Object.keys(I18N_CATALOG).length).toBeGreaterThan(595);
    for (const [key, entry] of Object.entries(I18N_CATALOG)) {
      expect(entry.en.trim(), key).not.toBe("");
      expect(entry.zh.trim(), key).not.toBe("");
      expect(placeholderNames(entry.zh), key).toEqual(placeholderNames(entry.en));
    }
    expect(translateCatalogText("Type {path} to confirm deletion.", "zh"))
      .toBe("输入 {path} 以确认删除。");
  });

  it("switches locale and persists the preference", async () => {
    localStorage.clear();
    const view = render(<I18nProvider><LocaleHarness /></I18nProvider>);
    expect(screen.getByText("Welcome back")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Toggle" }));
    expect(screen.getByText("欢迎回来")).toBeInTheDocument();
    await waitFor(() => expect(localStorage.getItem("gity.locale")).toBe("zh"));
    expect(document.documentElement.lang).toBe("zh-CN");
    view.unmount();
    render(<I18nProvider><LocaleHarness /></I18nProvider>);
    expect(screen.getByText("zh")).toBeInTheDocument();
    expect(screen.getByText("欢迎回来")).toBeInTheDocument();
  });
});
