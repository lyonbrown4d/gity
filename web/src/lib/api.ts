import axios, { type AxiosRequestConfig } from "axios";
import { clearTokens, getTokens, setTokens } from "@/lib/auth-store";

const API_BASE = ((import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "/api/v1").replace(/\/$/, "");

const apiClient = axios.create({
  baseURL: API_BASE,
  validateStatus: () => true,
});

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

export interface ApiResponseEnvelope<T> {
  code: number;
  message: string;
  data: T;
}

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const isApiResponseEnvelope = <T>(value: unknown): value is ApiResponseEnvelope<T> => {
  if (!isObject(value)) {
    return false;
  }
  return typeof value.code === "number" && typeof value.message === "string" && "data" in value;
};

const unwrapApiData = <T>(payload: unknown): T => {
  if (isApiResponseEnvelope<T>(payload)) {
    return payload.data;
  }
  return payload as T;
};

const errorMessageFromPayload = (payload: unknown, status: number): string => {
  if (typeof payload === "string" && payload.trim().length > 0) {
    return payload;
  }
  if (isApiResponseEnvelope(payload)) {
    return payload.message || `Request failed with ${status}`;
  }
  if (isObject(payload) && typeof payload.message === "string" && payload.message.trim().length > 0) {
    return payload.message;
  }
  return `Request failed with ${status}`;
};

const normalizeHeaders = (headersInit?: HeadersInit): Record<string, string> => {
  if (!headersInit) {
    return {};
  }

  if (headersInit instanceof Headers) {
    return Object.fromEntries(headersInit.entries());
  }

  if (Array.isArray(headersInit)) {
    return Object.fromEntries(headersInit);
  }

  return { ...(headersInit as Record<string, string>) };
};

const resolveRequestBody = (
  body: RequestInit["body"],
  headers: Record<string, string>,
): unknown => {
  if (body === undefined || body === null) {
    return undefined;
  }

  const isFormData = typeof FormData !== "undefined" && body instanceof FormData;
  if (isFormData) {
    return body;
  }

  if (typeof body !== "string") {
    return body;
  }

  const contentType = (headers["Content-Type"] ?? headers["content-type"] ?? "").toLowerCase();
  if (!contentType.includes("application/json")) {
    return body;
  }

  try {
    return JSON.parse(body);
  } catch {
    return body;
  }
};

const requestOnce = async (
  path: string,
  init?: RequestInit,
  options?: { auth?: boolean },
) => {
  const headers = normalizeHeaders(init?.headers);
  const auth = options?.auth ?? true;
  const isFormDataBody = typeof FormData !== "undefined" && init?.body instanceof FormData;

  if (!isFormDataBody && !headers["Content-Type"] && !headers["content-type"]) {
    headers["Content-Type"] = "application/json";
  }

  if (auth) {
    const tokens = getTokens();
    if (tokens) {
      headers.Authorization = `Bearer ${tokens.accessToken}`;
    }
  }

  const config: AxiosRequestConfig = {
    url: path,
    method: (init?.method ?? "GET") as AxiosRequestConfig["method"],
    headers,
    data: resolveRequestBody(init?.body, headers),
  };

  return apiClient.request(config);
};

const tryRefreshToken = async (): Promise<boolean> => {
  const tokens = getTokens();
  if (!tokens) {
    return false;
  }

  const response = await requestOnce(
    "/auth/refresh",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ refresh_token: tokens.refreshToken }),
    },
    { auth: false },
  );

  if (response.status < 200 || response.status >= 300) {
    clearTokens();
    return false;
  }

  const data = unwrapApiData<AuthResponse>(response.data);
  setTokens({ accessToken: data.token, refreshToken: data.refresh_token });
  return true;
};

export async function apiRequest<T>(
  path: string,
  init?: RequestInit,
  options?: { auth?: boolean; retryOnAuth?: boolean },
): Promise<T> {
  const auth = options?.auth ?? true;
  const retryOnAuth = options?.retryOnAuth ?? true;

  let response = await requestOnce(path, init, { auth });
  if (response.status === 401 && auth && retryOnAuth) {
    const refreshed = await tryRefreshToken();
    if (refreshed) {
      response = await requestOnce(path, init, { auth });
    }
  }

  if (response.status < 200 || response.status >= 300) {
    throw new Error(errorMessageFromPayload(response.data, response.status));
  }

  if (response.status === 204 || response.data === undefined) {
    return undefined as T;
  }

  return unwrapApiData<T>(response.data);
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
