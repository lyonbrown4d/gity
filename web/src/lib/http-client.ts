import axios from "axios";
import { getApiBaseUrl } from "./api";
import { getTokens, clearTokens, setTokens, type AuthTokens } from "./auth-store";

export interface RequestOptions extends RequestInit {
  auth?: boolean;
  skipRefresh?: boolean;
}
export interface HttpError extends Error {
  statusCode: number;
  status: number;
  response: { status: number; data: unknown };
}
const client = axios.create({ validateStatus: () => true, timeout: 30_000 });
const requestURL = (path: string): string => {
  const base = getApiBaseUrl();
  const suffix = path.replace(/^\/api\/v1(?=\/|\?|$)/, "");
  if (/^https?:\/\//i.test(path)) {
    const origin = typeof window === "undefined" ? "http://local.placeholder" : window.location.origin;
    if (new URL(path).origin !== new URL(base, origin).origin) throw new Error("Untrusted API origin");
    return path;
  }
  return `${base}${suffix.startsWith("/") || suffix.startsWith("?") ? "" : "/"}${suffix}`;
};
const message = (data: unknown, status: number): string => {
  if (data && typeof data === "object" && "message" in data && typeof data.message === "string") return data.message;
  return typeof data === "string" && data ? data : `Request failed with ${status}`;
};
const refresh = async (tokens: AuthTokens): Promise<AuthTokens | null> => {
  try {
    const { postApiV1AuthRefresh } = await import("./generated/auth/auth");
    const response = await postApiV1AuthRefresh({ refresh_token: tokens.refreshToken }, { auth: false, skipRefresh: true });
    const data = response.data;
    if (!data?.token || !data.refresh_token) throw new Error("Invalid session response");
    if (getTokens()?.refreshToken !== tokens.refreshToken) return getTokens();
    const next = { accessToken: data.token, refreshToken: data.refresh_token };
    setTokens(next);
    return next;
  } catch {
    if (getTokens()?.refreshToken === tokens.refreshToken) clearTokens();
    return null;
  }
};
let refreshInFlight: Promise<AuthTokens | null> | null = null;
const send = async (url: string, init: RequestOptions, tokens: AuthTokens | null) => {
  const headers = new Headers(init.headers);
  if (init.auth !== false && tokens) headers.set("Authorization", `Bearer ${tokens.accessToken}`);
  if (init.auth === false) headers.delete("Authorization");
  try { headers.set("Accept-Language", localStorage.getItem("gity.locale") === "zh" ? "zh" : "en"); } catch { /* Storage may be disabled. */ }
  let data: unknown = init.body;
  if (data instanceof FormData) headers.delete("Content-Type");
  else if (typeof data === "string" && headers.get("Content-Type")?.includes("application/json")) data = JSON.parse(data);
  return client.request({
    url: requestURL(url), method: init.method ?? "GET", headers: Object.fromEntries(headers.entries()), data,
    signal: init.signal ?? undefined, withCredentials: init.credentials === "include",
  });
};
export const requestEnvelope = async <T>(url: string, init: RequestOptions = {}): Promise<T> => {
  const tokens = init.auth === false ? null : getTokens();
  let response = await send(url, init, tokens);
  if (response.status === 401 && tokens && !init.skipRefresh) {
    let renewed = getTokens();
    if (renewed?.accessToken === tokens.accessToken) {
      if (!refreshInFlight) refreshInFlight = refresh(tokens).finally(() => { refreshInFlight = null; });
      renewed = await refreshInFlight;
    }
    if (renewed && !init.signal?.aborted) response = await send(url, init, renewed);
  }
  if (response.status < 200 || response.status >= 300) {
    if (response.status === 401 && init.auth !== false && getTokens()?.accessToken === tokens?.accessToken) clearTokens();
    throw Object.assign(new Error(message(response.data, response.status)), {
      statusCode: response.status, status: response.status, response: { status: response.status, data: response.data },
    } satisfies Pick<HttpError, "statusCode" | "status" | "response">);
  }
  return response.status === 204 ? undefined as T : response.data as T;
};
