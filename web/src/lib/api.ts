import { clearTokens, getTokens, setTokens } from "@/lib/auth-store";

const API_BASE = ((import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "/api/v1").replace(/\/$/, "");

export function getApiBaseUrl(): string {
  return API_BASE;
}

export interface LoginPayload {
  username: string;
  password: string;
}

export interface AuthResponse {
  user_id: string;
  username: string;
  organization_id?: string;
  organization_name?: string;
  token: string;
  refresh_token: string;
}

function toApiUrl(path: string): string {
  if (path.startsWith("http://") || path.startsWith("https://")) {
    return path;
  }
  if (path.startsWith("/")) {
    return `${API_BASE}${path}`;
  }
  return `${API_BASE}/${path}`;
}

async function rawFetch(path: string, init?: RequestInit): Promise<Response> {
  return fetch(toApiUrl(path), init);
}

async function tryRefreshToken(): Promise<boolean> {
  const tokens = getTokens();
  if (!tokens) {
    return false;
  }
  const response = await rawFetch("/auth/refresh", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ refresh_token: tokens.refreshToken }),
  });
  if (!response.ok) {
    clearTokens();
    return false;
  }
  const data = (await response.json()) as AuthResponse;
  setTokens({ accessToken: data.token, refreshToken: data.refresh_token });
  return true;
}

export async function apiRequest<T>(
  path: string,
  init?: RequestInit,
  options?: { auth?: boolean; retryOnAuth?: boolean },
): Promise<T> {
  const auth = options?.auth ?? true;
  const retryOnAuth = options?.retryOnAuth ?? true;
  const headers = new Headers(init?.headers ?? {});
  headers.set("Content-Type", "application/json");

  if (auth) {
    const tokens = getTokens();
    if (tokens) {
      headers.set("Authorization", `Bearer ${tokens.accessToken}`);
    }
  }

  const response = await rawFetch(path, { ...init, headers });

  if (response.status === 401 && auth && retryOnAuth) {
    const refreshed = await tryRefreshToken();
    if (refreshed) {
      return apiRequest<T>(path, init, { auth, retryOnAuth: false });
    }
  }

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Request failed with ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }
  return (await response.json()) as T;
}

export async function login(payload: LoginPayload): Promise<AuthResponse> {
  const response = await apiRequest<AuthResponse>(
    "/auth/login",
    {
      method: "POST",
      body: JSON.stringify(payload),
    },
    { auth: false },
  );
  setTokens({
    accessToken: response.token,
    refreshToken: response.refresh_token,
  });
  return response;
}

export async function logout(): Promise<void> {
  const tokens = getTokens();
  if (!tokens) {
    return;
  }
  try {
    await apiRequest<void>("/auth/logout", {
      method: "POST",
      body: JSON.stringify({ refresh_token: tokens.refreshToken }),
    });
  } finally {
    clearTokens();
  }
}
