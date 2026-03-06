import DOMPurify from "dompurify";
import { marked } from "marked";

const LANGUAGE_COLORS: Record<string, string> = {
  rust: "#dea584",
  typescript: "#3178c6",
  javascript: "#f1e05a",
  go: "#00add8",
  java: "#b07219",
  python: "#3572a5",
  shell: "#89e051",
  html: "#e34c26",
  css: "#563d7c",
  json: "#6e4a7e",
  markdown: "#083fa1",
  toml: "#9c4221",
  yaml: "#cb171e",
};

export const shortSha = (value: string): string => {
  return value.slice(0, 8);
};

export const formatTime = (value: string): string => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString();
};

export const extractErrorMessage = (error: unknown): string => {
  if (!(error instanceof Error)) {
    return "Unknown error";
  }
  const raw = error.message.trim();
  if (!raw) {
    return "Unknown error";
  }
  try {
    const parsed = JSON.parse(raw) as { message?: string };
    if (typeof parsed.message === "string" && parsed.message.trim().length > 0) {
      return parsed.message;
    }
  } catch {
    // ignore non-json message
  }
  return raw;
};

export const detectLanguage = (path: string): string => {
  const file = path.split("/").pop()?.toLowerCase() ?? "";
  if (file.endsWith(".rs")) {
    return "rust";
  }
  if (file.endsWith(".ts") || file.endsWith(".tsx")) {
    return "typescript";
  }
  if (file.endsWith(".js") || file.endsWith(".jsx")) {
    return "javascript";
  }
  if (file.endsWith(".json")) {
    return "json";
  }
  if (file.endsWith(".md")) {
    return "markdown";
  }
  if (file.endsWith(".toml")) {
    return "ini";
  }
  if (file.endsWith(".yml") || file.endsWith(".yaml")) {
    return "yaml";
  }
  if (file.endsWith(".go")) {
    return "go";
  }
  if (file.endsWith(".java")) {
    return "java";
  }
  if (file.endsWith(".py")) {
    return "python";
  }
  if (file.endsWith(".sh")) {
    return "shell";
  }
  return "plaintext";
};

export const languageBarColor = (language: string): string => {
  const normalized = language.trim().toLowerCase();
  return LANGUAGE_COLORS[normalized] ?? "#6b7280";
};

export const formatBytes = (value: number): string => {
  if (!Number.isFinite(value) || value <= 0) {
    return "0 B";
  }
  const units = ["B", "KB", "MB", "GB"];
  let size = value;
  let index = 0;
  while (size >= 1024 && index < units.length - 1) {
    size /= 1024;
    index += 1;
  }
  const precision = size >= 100 || index === 0 ? 0 : 1;
  return `${size.toFixed(precision)} ${units[index]}`;
};

export const normalizeRepoFilePath = (path: string): string => {
  return path.trim().replace(/\\/g, "/").replace(/^\/+|\/+$/g, "");
};

export const renderMarkdown = async (content: string): Promise<string> => {
  const html = await marked.parse(content);
  return DOMPurify.sanitize(html);
};
