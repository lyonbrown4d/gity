import type { customFetch } from "@/lib/orval-fetch";
import { generatedOperations } from "@/lib/generated/refine-registry";
import { getApiBaseUrl } from "@/lib/api";

export interface OperationInput {
  path: Record<string, string>;
  query: Record<string, string | string[]>;
  payload: unknown;
  options: NonNullable<Parameters<typeof customFetch>[1]> & { auth?: boolean; skipRefresh?: boolean };
}
export interface GeneratedOperation {
  method: string;
  path: string;
  pattern: RegExp;
  names: string[];
  queryNames: string[];
  acceptsQuery: boolean;
  invoke: (input: OperationInput) => Promise<unknown>;
}
interface RequestOptions {
  method?: string;
  payload?: unknown;
  headers?: HeadersInit;
  auth?: boolean;
  skipRefresh?: boolean;
  signal?: AbortSignal;
}
const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);
const multipartPayload = (payload: unknown): unknown => {
  if (!(typeof FormData !== "undefined" && payload instanceof FormData)) return payload;
  const result: Record<string, FormDataEntryValue | FormDataEntryValue[]> = {};
  for (const key of new Set(payload.keys())) {
    const values = payload.getAll(key);
    result[key] = values.length === 1 ? values[0] : values;
  }
  return result;
};
const normalizedURL = (path: string): URL => {
  const configured = getApiBaseUrl();
  const origin = typeof window === "undefined" ? "http://local.placeholder" : window.location.origin;
  const base = new URL(configured, origin);
  const request = new URL(path, base.origin);
  if (request.origin !== base.origin) throw new Error("Refine only dispatches operations to the configured API origin");
  return request;
};
const findOperation = (
  method: string,
  candidates: string[],
): { operation: GeneratedOperation; values: string[] } | null => {
  for (const operation of generatedOperations) {
    if (operation.method !== method) continue;
    for (const candidate of candidates) {
      const match = operation.pattern.exec(candidate);
      if (match) return { operation, values: match.slice(1) };
    }
  }
  return null;
};
export const generatedRequest = async <T>(path: string, options: RequestOptions = {}): Promise<T> => {
  const url = normalizedURL(path);
  const method = (options.method ?? "GET").toUpperCase();
  const candidates = [url.pathname, `/api/v1${url.pathname}`];
  const found = findOperation(method, candidates);
  if (!found) throw new Error(`No generated API operation supports ${method} ${url.pathname}`);
  const { operation, values } = found;
  if (!operation.acceptsQuery && url.searchParams.size)
    throw new Error(`The generated API operation ${method} ${operation.path} does not accept query parameters`);
  for (const key of url.searchParams.keys()) {
    if (!operation.queryNames.includes(key))
      throw new Error(`Unsupported query parameter "${key}" for ${method} ${operation.path}`);
  }
  const query: OperationInput["query"] = {};
  for (const key of new Set(url.searchParams.keys())) {
    const values = url.searchParams.getAll(key);
    query[key] = values.length === 1 ? values[0] : values;
  }
  const response = await operation.invoke({
    path: Object.fromEntries(operation.names.map((name, index) => [name, values[index]])),
    query,
    payload: multipartPayload(options.payload),
    options: { headers: options.headers, signal: options.signal, auth: options.auth, skipRefresh: options.skipRefresh },
  });
  if (
    record(response) &&
    typeof response.code === "number" &&
    typeof response.message === "string" &&
    "data" in response
  )
    return response.data as T;
  return response as T;
};
