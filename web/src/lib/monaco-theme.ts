import type { Monaco } from "@monaco-editor/react";
import { colord } from "colord";

const subscriptions = new WeakSet<Monaco>();

const semanticColor = (name: string): string => {
  const probe = document.createElement("span");
  probe.style.color = `hsl(var(${name}))`;
  document.body.append(probe);
  const color = window.getComputedStyle(probe).color;
  probe.remove();
  const channels = color.match(/[\d.]+/g)?.map(Number) ?? [];
  if (channels.length < 3) return "#000000";
  return `#${channels.slice(0, 3).map((channel) => Math.round(channel).toString(16).padStart(2, "0")).join("")}`;
};

export const registerEditorTheme = (monaco: Monaco): void => {
  const background = semanticColor("--background");
  const isDark = colord(background).isDark();
  const foreground = semanticColor("--foreground");
  const muted = semanticColor("--muted");
  const mutedForeground = semanticColor("--muted-foreground");
  const border = semanticColor("--border");
  const primary = semanticColor("--primary");
  const destructive = semanticColor("--destructive");
  const success = semanticColor("--success");

  const definition = {
    base: isDark ? "vs-dark" : "vs",
    inherit: true,
    rules: [],
    colors: {
      "editor.background": background,
      "editor.foreground": foreground,
      "editorLineNumber.foreground": mutedForeground,
      "editorLineNumber.activeForeground": foreground,
      "editor.lineHighlightBackground": muted,
      "editor.selectionBackground": `${primary}40`,
      "editorCursor.foreground": primary,
      "editorGutter.background": background,
      "editorWidget.background": background,
      "editorWidget.border": border,
      "diffEditor.insertedTextBackground": `${success}26`,
      "diffEditor.removedTextBackground": `${destructive}26`,
    },
  } as const;
  monaco.editor.defineTheme("gity-light", definition);
  monaco.editor.defineTheme("gity-dark", definition);
  monaco.editor.setTheme(document.documentElement.classList.contains("dark") ? "gity-dark" : "gity-light");
  if (!subscriptions.has(monaco)) {
    subscriptions.add(monaco);
    new MutationObserver(() => registerEditorTheme(monaco)).observe(document.documentElement, {
      attributes: true, attributeFilter: ["class", "style", "data-theme"],
    });
  }
};
