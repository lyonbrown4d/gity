const normalizePath = (value: string): string => {
  const path = value.trim();
  if (path === "" || path === "/") {
    return "/api/v1";
  }

  if (path === "/api") {
    return "/api/v1";
  }

  if (path === "/v1" || path === "/api/v1") {
    return "/api/v1";
  }

  if (path.startsWith("/api/")) {
    return path;
  }

  return "/api/v1";
};

const normalizeApiBase = (raw?: string): string => {
  const trimmed = (raw ?? "").trim().replace(/\/+$/, "");
  if (trimmed === "") {
    return "/api/v1";
  }

  const isAbsolute = /^https?:\/\//i.test(trimmed);
  if (!isAbsolute) {
    return normalizePath(trimmed);
  }

  try {
    const parsed = new URL(trimmed);
    const normalizedPath = normalizePath(parsed.pathname || "/");
    parsed.pathname = normalizedPath;
    return parsed.toString().replace(/\/+$/, "");
  } catch {
    return normalizePath(trimmed);
  }
};

const isDev = Boolean(import.meta.env.DEV);
const forceAbsoluteApi = String((import.meta.env as { VITE_FORCE_ABSOLUTE_API?: string }).VITE_FORCE_ABSOLUTE_API ?? "")
  .toLowerCase() === "true";

const API_BASE = isDev && !forceAbsoluteApi
  ? "/api/v1"
  : normalizeApiBase((import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "/api/v1");

export const getApiBaseUrl = (): string => API_BASE;
