import type { IssueEditorToolbarAction } from "./issue-markdown-toolbar";

export type MilkdownCommandKey =
  | "ToggleStrong"
  | "ToggleEmphasis"
  | "ToggleStrikethrough"
  | "ToggleInlineCode"
  | "WrapInHeading"
  | "WrapInBlockquote"
  | "WrapInBulletList"
  | "WrapInOrderedList"
  | "CreateCodeBlock"
  | "ToggleLink"
  | "InsertImage";

export type ToolbarRequest =
  | { type: "insert_text"; text: string }
  | { type: "command"; key: MilkdownCommandKey; payload?: unknown };

export function appendMarkdown(source: string, snippet: string): string {
  const prefix = source.trimEnd().length === 0 ? "" : "\n\n";
  return `${source}${prefix}${snippet}`;
}

export function resolveToolbarRequest(
  action: IssueEditorToolbarAction,
  t: (text: string) => string,
): ToolbarRequest | null {
  if (action === "bold") {
    return { type: "command", key: "ToggleStrong" };
  }
  if (action === "italic") {
    return { type: "command", key: "ToggleEmphasis" };
  }
  if (action === "strike") {
    return { type: "command", key: "ToggleStrikethrough" };
  }
  if (action === "inline-code") {
    return { type: "command", key: "ToggleInlineCode" };
  }
  if (action === "heading1") {
    return { type: "command", key: "WrapInHeading", payload: 1 };
  }
  if (action === "heading2") {
    return { type: "command", key: "WrapInHeading", payload: 2 };
  }
  if (action === "quote") {
    return { type: "command", key: "WrapInBlockquote" };
  }
  if (action === "bullet-list") {
    return { type: "command", key: "WrapInBulletList" };
  }
  if (action === "ordered-list") {
    return { type: "command", key: "WrapInOrderedList" };
  }
  if (action === "code-block") {
    return { type: "command", key: "CreateCodeBlock" };
  }
  if (action === "issue-ref") {
    return { type: "insert_text", text: "#123" };
  }
  if (action === "link") {
    const href = window.prompt(t("Enter link URL"), "https://")?.trim();
    if (!href) {
      return null;
    }
    return { type: "command", key: "ToggleLink", payload: { href } };
  }
  const src = window.prompt(t("Enter image URL"), "https://")?.trim();
  if (!src) {
    return null;
  }
  const alt = window.prompt(t("Image description (optional)"), "")?.trim();
  return { type: "command", key: "InsertImage", payload: { src, alt } };
}
