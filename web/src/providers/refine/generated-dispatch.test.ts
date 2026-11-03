import { beforeEach, describe, expect, it, vi } from "vitest";
import { dataProvider } from "../data-provider";
import { customFetch } from "@/lib/orval-fetch";
vi.mock("@/lib/api", () => ({ getApiBaseUrl: () => "/api/v1" }));
vi.mock("@/lib/orval-fetch", () => ({ customFetch: vi.fn() }));
const transport = vi.mocked(customFetch);
beforeEach(() => {
  transport.mockReset();
  transport.mockResolvedValue({ code: 0, message: "ok", data: { id: "result" } });
});
describe("Refine dispatch through generated Orval request functions", () => {
  it("preserves Snowflake strings and lets the generated function serialize query parameters", async () => {
    const response = await dataProvider.custom!({
      url: "/projects/9223372036854775806/repository/blob",
      method: "get",
      query: { branch_name: "feature/ci", path: "src/main.rs" },
    });
    expect(response.data).toEqual({ id: "result" });
    const [url, options] = transport.mock.calls[0];
    expect(url).toBe(
      "/api/v1/projects/9223372036854775806/repository/blob?branch_name=feature%2Fci&path=src%2Fmain.rs",
    );
    expect(options?.method).toBe("GET");
  });
  it("uses generated mutation serialization and passes per-request transport settings", async () => {
    const payload = { ref_name: "main", commit_sha: "saved-sha", config_source: ".gity-ci.yml", config_content: "" };
    await dataProvider.custom!({
      url: "/projects/41/pipelines",
      method: "post",
      payload,
      headers: { "X-Request-ID": "request" },
      meta: { auth: false, skipRefresh: true },
    });
    const [url, options] = transport.mock.calls[0];
    expect(url).toBe("/api/v1/projects/41/pipelines");
    expect(options?.method).toBe("POST");
    expect(options?.body).toBe(JSON.stringify(payload));
    expect(options).toMatchObject({ auth: false, skipRefresh: true });
    expect(new Headers(options?.headers).get("X-Request-ID")).toBe("request");
  });
  it("preserves generated multipart behavior and unwrapped response contracts", async () => {
    const body = new FormData();
    body.set("file", new Blob(["attachment"], { type: "text/plain" }), "note.txt");
    transport.mockResolvedValueOnce({ id: "attachment", url: "/file" });
    const result = await dataProvider.custom!({
      url: "/projects/41/issues/attachments",
      method: "post",
      payload: body,
    });
    expect(result.data).toEqual({ id: "attachment", url: "/file" });
    const [, options] = transport.mock.calls[0];
    expect(options?.body).toBeInstanceOf(FormData);
    expect((options?.body as FormData).get("file")).toBeInstanceOf(Blob);
  });
  it("routes known resource adapters through generated operations and rejects unsupported operations", async () => {
    transport.mockResolvedValueOnce({ code: 0, message: "ok", data: [{ id: "41", name: "project" }] });
    const list = await dataProvider.getList({ resource: "projects", meta: { organization_id: "31" } });
    expect(list.data).toEqual([{ id: "41", name: "project" }]);
    expect(String(transport.mock.calls[0][0])).toContain("/api/v1/projects?");
    expect(String(transport.mock.calls[0][0])).toContain("organization_id=31");
    transport.mockClear();
    await expect(
      dataProvider.custom!({ url: "/projects/41/unknown-operation", method: "post", payload: {} }),
    ).rejects.toThrow("No generated API operation");
    await expect(dataProvider.getList({ resource: "invented-resource" })).rejects.toThrow("not supported");
    await expect(
      dataProvider.custom!({ url: "https://untrusted.example/api/v1/projects", method: "get" }),
    ).rejects.toThrow("configured API origin");
    expect(transport).not.toHaveBeenCalled();
  });
  it("preserves authorization and conflict errors for Refine handling", async () => {
    const conflict = Object.assign(new Error("branch changed"), { statusCode: 409 });
    transport.mockRejectedValueOnce(conflict);
    await expect(dataProvider.custom!({ url: "/projects/41/pipelines", method: "post", payload: {} })).rejects.toBe(
      conflict,
    );
  });
});
