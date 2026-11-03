import { beforeEach, describe, expect, it, vi } from "vitest";
import { customFetch } from "@/lib/orval-fetch";
import { dataProvider } from "../data-provider";
vi.mock("@/lib/api", () => ({ getApiBaseUrl: () => "/api/v1" }));
vi.mock("@/lib/orval-fetch", () => ({ customFetch: vi.fn() }));
const transport = vi.mocked(customFetch);
const rows = [
  { id: "1", name: "C" },
  { id: "2", name: "A" },
  { id: "3", name: "B" },
];
beforeEach(() => {
  transport.mockReset();
  transport.mockResolvedValue({ code: 0, message: "ok", data: rows });
});
describe("Resource list adapters against generated request contracts", () => {
  it("uses the parameter-free generated organizations request and applies explicit local pagination and sorting", async () => {
    const result = await dataProvider.getList({
      resource: "organizations",
      pagination: { mode: "server", currentPage: 2, pageSize: 1 },
      sorters: [{ field: "name", order: "asc" }],
    });
    expect(transport.mock.calls[0][0]).toBe("/api/v1/orgs");
    expect(result).toEqual({ data: [{ id: "3", name: "B" }], total: 3 });
    const all = await dataProvider.getList({ resource: "organizations", pagination: { mode: "off" } });
    expect(all).toEqual({ data: rows, total: 3 });
    expect(transport.mock.calls[1][0]).toBe("/api/v1/orgs");
  });
  it.each(["projects", "my-projects", "repositories", "my-repositories"])(
    "sends only the documented organization filter for %s",
    async (resource) => {
      const result = await dataProvider.getList({
        resource,
        meta: { organization_id: "31" },
        filters: [{ field: "id", operator: "in", value: ["2", "3"] }],
        pagination: { mode: "server", currentPage: 2, pageSize: 1 },
      });
      expect(transport.mock.calls[0][0]).toBe("/api/v1/projects?organization_id=31");
      expect(result).toEqual({ data: [{ id: "3", name: "B" }], total: 2 });
    },
  );
  it("keeps members/profile requests query-free and paginates their returned collections locally", async () => {
    const members = await dataProvider.getList({
      resource: "organization-members",
      meta: { organization_id: "31" },
      pagination: { mode: "server", currentPage: 2, pageSize: 1 },
    });
    expect(transport.mock.calls[0][0]).toBe("/api/v1/orgs/31/members");
    expect(members).toEqual({ data: [rows[1]], total: 3 });
    transport.mockResolvedValueOnce({ code: 0, message: "ok", data: rows[0] });
    const profile = await dataProvider.getList({
      resource: "profile",
      pagination: { mode: "server", currentPage: 2, pageSize: 1 },
    });
    expect(transport.mock.calls[1][0]).toBe("/api/v1/users/me");
    expect(profile).toEqual({ data: [], total: 1 });
  });
  it("uses Refine currentPage and the real users page-size limit for server pagination", async () => {
    transport.mockResolvedValueOnce({
      code: 0,
      message: "ok",
      data: { items: rows, total: 150, page: 2, page_size: 100 },
    });
    const result = await dataProvider.getList({
      resource: "users",
      pagination: { mode: "server", currentPage: 2, pageSize: 300 },
    });
    expect(transport.mock.calls[0][0]).toBe("/api/v1/users?page=2&page_size=100");
    expect(result).toEqual({ data: rows, total: 150 });
  });
  it("loads requested user identities directly rather than dropping IDs outside the first directory page", async () => {
    transport.mockImplementation(async (url) => ({ code: 0, message: "ok", data: { id: url.split("/").pop() } }));
    const result = await dataProvider.getMany!({ resource: "users", ids: ["200", "9223372036854775806"] });
    expect(result.data).toEqual([{ id: "200" }, { id: "9223372036854775806" }]);
    expect(transport.mock.calls.map(([url]) => url)).toEqual([
      "/api/v1/users/200",
      "/api/v1/users/9223372036854775806",
    ]);
  });
  it("rejects unsupported business filters, conflicting filters and unknown custom query keys before transport", async () => {
    await expect(
      dataProvider.getList({
        resource: "organizations",
        filters: [{ field: "visibility", operator: "eq", value: "private" }],
      }),
    ).rejects.toThrow("Unsupported list filter");
    await expect(dataProvider.getList({ resource: "projects", meta: { all: true } })).rejects.toThrow("all filter");
    await expect(
      dataProvider.getList({
        resource: "projects",
        meta: { organization_id: "31" },
        filters: [{ field: "organization_id", operator: "eq", value: "32" }],
      }),
    ).rejects.toThrow("Conflicting organization_id");
    await expect(dataProvider.custom!({ url: "/projects?visibility=private", method: "get" })).rejects.toThrow(
      "Unsupported query parameter",
    );
    await expect(
      dataProvider.getList({
        resource: "projects",
        filters: [
          { field: "organization_id", operator: "eq", value: "31" },
          { field: "organization_id", operator: "eq", value: "32" },
        ],
      }),
    ).rejects.toThrow("Conflicting list filters");
    expect(transport).not.toHaveBeenCalled();
  });
});
