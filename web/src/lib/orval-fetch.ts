import { getApiBaseUrl } from "@/lib/api";
import { clearTokens, getTokens, setTokens, type AuthTokens } from "@/lib/auth-store";

/**
 * Orval mutator: the single HTTP entry point for generated clients.
 *
 * - uses the same `/api/v1` base as the hand-written axios layer, including the
 *   Vite dev proxy and `VITE_API_BASE_URL`;
 * - attaches the bearer access token from `auth-store`;
 * - refreshes once on `401` (single-flight) and retries the original request;
 * - throws `Error(message)` for non-2xx responses so Refine/React Query error
 *   handling (`extractErrorMessage`) keeps working;
 * - returns the raw `{ code, message, data }` envelope. Generated hooks are
 *   typed against that envelope; unwrap `response.data` in the page.
 */
const API_BASE = getApiBaseUrl();

const isAbsoluteUrl = (value: string): boolean => /^https?:\/\//i.test(value);

const buildUrl = (path: string): string => {
  if (isAbsoluteUrl(path) || !isAbsoluteUrl(API_BASE)) {
    return path;
  }
  const base = new URL(API_BASE);
  const target = new URL(path, base.origin);
  const suffix = target.pathname.replace(/^\/api\/v1/, "");
  return `${base.origin}${base.pathname}${suffix}${target.search}`;
};

const parseBody = async (response: Response): Promise<unknown> => {
  if ([204, 205, 304].includes(response.status)) {
    return null;
  }
  const text = await response.text();
  if (!text) {
    return null;
  }
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return text;
  }
};

const errorMessageFrom = (payload: unknown, status: number): string => {
  if (typeof payload === "string" && payload.trim().length > 0) {
    return payload;
  }
  if (payload && typeof payload === "object") {
    const message = (payload as { message?: unknown }).message;
    if (typeof message === "string" && message.trim().length > 0) {
      return message;
    }
  }
  return `Request failed with ${status}`;
};

let refreshInFlight: Promise<AuthTokens | null> | null = null;

const refreshTokens = async (): Promise<AuthTokens | null> => {
  if (refreshInFlight) {
    return refreshInFlight;
  }
  const tokens = getTokens();
  if (!tokens) {
    return null;
  }
  refreshInFlight = (async () => {
    try {
      const response = await fetch(buildUrl("/api/v1/auth/refresh"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: tokens.refreshToken }),
      });
      const payload = await parseBody(response);
      const data = payload && typeof payload === "object"
        ? (payload as { data?: { token?: unknown; refresh_token?: unknown } }).data
        : undefined;
      if (!response.ok || typeof data?.token !== "string" || typeof data.refresh_token !== "string") {
        clearTokens();
        return null;
      }
      const next: AuthTokens = { accessToken: data.token, refreshToken: data.refresh_token };
      setTokens(next);
      return next;
    } catch {
      clearTokens();
      return null;
    } finally {
      refreshInFlight = null;
    }
  })();
  return refreshInFlight;
};

export const customFetch = async <T>(url: string, init: RequestInit = {}): Promise<T> => {
  const send = async (tokens: AuthTokens | null): Promise<Response> => {
    const headers = new Headers(init.headers);
    if (tokens) {
      headers.set("Authorization", `Bearer ${tokens.accessToken}`);
    }
    return fetch(buildUrl(url), { ...init, headers });
  };

  let response = await send(getTokens());
  if (response.status === 401) {
    const refreshed = await refreshTokens();
    if (refreshed) {
      response = await send(refreshed);
    }
  }

  const payload = await parseBody(response);
  if (!response.ok) {
    if (response.status === 401) {
      clearTokens();
    }
    throw new Error(errorMessageFrom(payload, response.status));
  }
  return payload as T;
};

export default customFetch;
