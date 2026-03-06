import type {
  BaseRecord,
  CreateParams,
  DeleteOneParams,
  GetListParams,
  GetManyParams,
  GetOneParams,
  UpdateParams,
} from "@refinedev/core";
import { apiRequest } from "@/lib/api";
import {
  asRecord,
  asString,
  resolveBoolean,
  resolveLimit,
  resolveOrganizationId,
  type MetaLike,
  type ResourceAdapter,
} from "./shared";

const appendIds = (query: URLSearchParams, ids: Array<string | number>): void => {
  if (ids.length > 0) {
    query.set("ids", ids.map((item) => String(item)).join(","));
  }
};

const resolveFilterIds = (params: GetListParams): string[] =>
  params.filters
    ?.filter((item) => "field" in item && item.field === "id" && item.operator === "in")
    .flatMap((item) => (Array.isArray(item.value) ? item.value : []))
    .map((item) => String(item)) ?? [];

const organizationsAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    const query = new URLSearchParams();
    appendIds(query, resolveFilterIds(params));
    const path = query.size > 0 ? `/orgs?${query.toString()}` : "/orgs";
    const data = await apiRequest<TData[]>(path);
    return { data, total: data.length };
  },
  getOne: async <TData extends BaseRecord = BaseRecord>(params: GetOneParams) => ({
    data: await apiRequest<TData>(`/orgs/${params.id}`),
  }),
  getMany: async <TData extends BaseRecord = BaseRecord>(params: GetManyParams) => {
    const query = new URLSearchParams();
    appendIds(query, params.ids);
    const data = await apiRequest<TData[]>(`/orgs?${query.toString()}`);
    return { data };
  },
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: CreateParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>("/orgs", {
      method: "POST",
      body: JSON.stringify(params.variables),
    });
    return { data };
  },
  update: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: UpdateParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>(`/orgs/${params.id}`, {
      method: "PATCH",
      body: JSON.stringify(params.variables),
    });
    return { data };
  },
  deleteOne: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: DeleteOneParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>(`/orgs/${params.id}`, { method: "DELETE" });
    return { data };
  },
};

const repositoriesAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    const query = new URLSearchParams();
    const organizationId = resolveOrganizationId(params.meta as MetaLike, params.filters);
    if (organizationId) {
      query.set("organization_id", organizationId);
    }
    const all = resolveBoolean(params.meta as MetaLike, "all");
    if (all) {
      query.set("all", "true");
    }
    appendIds(query, resolveFilterIds(params));
    const path = query.size > 0 ? `/repos?${query.toString()}` : "/repos";
    const data = await apiRequest<TData[]>(path);
    return { data, total: data.length };
  },
  getOne: async <TData extends BaseRecord = BaseRecord>(params: GetOneParams) => ({
    data: await apiRequest<TData>(`/repos/${params.id}`),
  }),
  getMany: async <TData extends BaseRecord = BaseRecord>(params: GetManyParams) => {
    const query = new URLSearchParams();
    const organizationId = resolveOrganizationId(params.meta as MetaLike);
    if (organizationId) {
      query.set("organization_id", organizationId);
    }
    const all = resolveBoolean(params.meta as MetaLike, "all");
    if (all) {
      query.set("all", "true");
    }
    appendIds(query, params.ids);
    const data = await apiRequest<TData[]>(`/repos?${query.toString()}`);
    return { data };
  },
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: CreateParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>("/repos", {
      method: "POST",
      body: JSON.stringify(params.variables),
    });
    return { data };
  },
  deleteOne: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: DeleteOneParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>(`/repos/${params.id}`, { method: "DELETE" });
    return { data };
  },
};

const organizationMembersAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    const organizationId = resolveOrganizationId(params.meta as MetaLike, params.filters);
    if (!organizationId) {
      throw new Error("organization_id is required for organization-members");
    }
    const data = await apiRequest<TData[]>(`/orgs/${organizationId}/members`);
    return { data, total: data.length };
  },
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: CreateParams<TVariables>,
  ) => {
    const payload = asRecord(params.variables as unknown);
    const organizationId = asString(payload?.organization_id) ?? asString(payload?.organizationId);
    if (!organizationId) {
      throw new Error("organization_id is required for organization-members");
    }

    const data = await apiRequest<TData>(`/orgs/${organizationId}/members`, {
      method: "POST",
      body: JSON.stringify({
        user_id: payload?.user_id ?? payload?.userId,
        role: payload?.role,
      }),
    });
    return { data };
  },
};

const usersAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    const limit = resolveLimit(params.pagination, 100);
    const query = new URLSearchParams({ limit: String(limit) });
    appendIds(query, resolveFilterIds(params));
    const data = await apiRequest<TData[]>(`/users?${query.toString()}`);
    return { data, total: data.length };
  },
  getOne: async <TData extends BaseRecord = BaseRecord>(params: GetOneParams) => {
    if (String(params.id) === "me") {
      return { data: await apiRequest<TData>("/users/me") };
    }
    return { data: await apiRequest<TData>(`/users/${params.id}`) };
  },
  getMany: async <TData extends BaseRecord = BaseRecord>(params: GetManyParams) => {
    const query = new URLSearchParams();
    appendIds(query, params.ids);
    const data = await apiRequest<TData[]>(`/users?${query.toString()}`);
    return { data };
  },
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: CreateParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>("/users", {
      method: "POST",
      body: JSON.stringify(params.variables),
    });
    return { data };
  },
  update: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: UpdateParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>("/users/me", {
      method: "PATCH",
      body: JSON.stringify(params.variables),
    });
    return { data };
  },
};

const profileAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>() => {
    const me = await apiRequest<TData>("/users/me");
    return { data: [me], total: 1 };
  },
  getOne: async <TData extends BaseRecord = BaseRecord>() => ({
    data: await apiRequest<TData>("/users/me"),
  }),
  update: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: UpdateParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>("/users/me", {
      method: "PATCH",
      body: JSON.stringify(params.variables),
    });
    return { data };
  },
};

const resourceAdapters: Record<string, ResourceAdapter> = {
  organizations: organizationsAdapter,
  repositories: repositoriesAdapter,
  "my-repositories": repositoriesAdapter,
  "organization-members": organizationMembersAdapter,
  users: usersAdapter,
  profile: profileAdapter,
};

export const resolveAdapter = (resource: string): ResourceAdapter | undefined => resourceAdapters[resource];
