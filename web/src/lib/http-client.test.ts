import { beforeEach, describe, expect, it, vi } from "vitest";
const { request } = vi.hoisted(() => ({ request: vi.fn() }));
vi.mock("axios", () => ({ default: { create: () => ({ request }) } }));
import { requestEnvelope } from "./http-client";
import { getTokens, setTokens } from "./auth-store";

beforeEach(() => { request.mockReset(); localStorage.clear(); });
describe("single Orval Axios transport", () => {
  it("attaches request-local locale and keeps public authentication calls anonymous", async () => {
    setTokens({ accessToken: "private", refreshToken: "refresh" });
    localStorage.setItem("gity.locale", "zh");
    request.mockResolvedValue({ status: 200, data: { code: 0, data: "ok" } });
    await requestEnvelope("/api/v1/auth/login", { auth: false, method: "POST" });
    expect(request.mock.calls[0][0]).toMatchObject({ url: "/api/v1/auth/login", headers: { "accept-language": "zh" } });
    expect(request.mock.calls[0][0].headers.authorization).toBeUndefined();
  });
  it("refreshes once for concurrent expired requests through generated auth code", async () => {
    setTokens({ accessToken: "expired", refreshToken: "initial-refresh" });
    request.mockImplementation(async (config) => {
      if (config.url.endsWith("/auth/refresh")) return {
        status: 200, data: { code: 0, data: { token: "renewed", refresh_token: "renewed-refresh" } },
      };
      return config.headers.authorization === "Bearer expired"
        ? { status: 401, data: { message: "Expired" } }
        : { status: 200, data: { code: 0, data: "authorized" } };
    });
    const results = await Promise.all([requestEnvelope("/api/v1/users/me"), requestEnvelope("/api/v1/todos")]);
    expect(results).toEqual([{ code: 0, data: "authorized" }, { code: 0, data: "authorized" }]);
    expect(request.mock.calls.filter(([config]) => config.url.endsWith("/auth/refresh"))).toHaveLength(1);
    expect(getTokens()?.accessToken).toBe("renewed");
  });
  it("preserves HTTP conflict details for Refine error behavior", async () => {
    request.mockResolvedValue({ status: 409, data: { code: 409, message: "Branch changed" } });
    await expect(requestEnvelope("/api/v1/projects/1/repository/files", { method: "POST" }))
      .rejects.toMatchObject({ statusCode: 409, status: 409, message: "Branch changed" });
  });
  it("clears rejected sessions and prevents credential transmission to another origin", async () => {
    setTokens({ accessToken: "expired", refreshToken: "expired-refresh" });
    request.mockResolvedValue({ status: 401, data: { message: "Expired" } });
    await expect(requestEnvelope("/api/v1/users/me")).rejects.toMatchObject({ statusCode: 401 });
    expect(getTokens()).toBeNull();
    const calls = request.mock.calls.length;
    await expect(requestEnvelope("https://untrusted.invalid/api/v1/users/me")).rejects.toThrow("Untrusted API origin");
    expect(request.mock.calls).toHaveLength(calls);
  });
});
