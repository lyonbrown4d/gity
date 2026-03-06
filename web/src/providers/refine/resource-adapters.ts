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
  normalizeListPayload,
  type PagePayload,
  resolveBoolean,
  resolvePagination,
  resolveOrganizationId,
  type MetaLike,
  type ResourceAdapter,
} from "./shared";

type ListPayload<TData extends BaseRecord = BaseRecord> = TData[] | PagePayload<TData>;

const appendIds = (query: URLSearchParams, ids: Array<string | number>): void => {
  if (ids.length > 0) {
    query.set("ids", ids.map((item) => String(item)).join(","));
  }
};

const appendPagination = (
  query: URLSearchParams,
  pagination: GetListParams["pagination"],
  fallbackPageSize: number,
): void => {
  const { page, pageSize } = resolvePagination(pagination, 1, fallbackPageSize, 200);
  query.set("page", String(page));
  query.set("page_size", String(pageSize));
};

const resolveFilterIds = (params: GetListParams): string[] =>
  params.filters
    ?.filter((item) => "field" in item && item.field === "id" && item.operator === "in")
    .flatMap((item) => (Array.isArray(item.value) ? item.value : []))
    .map((item) => String(item)) ?? [];

const organizationsAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    const ids = resolveFilterIds(params);
    const query = new URLSearchParams();
    appendIds(query, ids);
    if (ids.length === 0) {
      appendPagination(query, params.pagination, 50);
    }
    const path = query.size > 0 ? `/orgs?${query.toString()}` : "/orgs";
    const payload = await apiRequest<ListPayload<TData>>(path);
    return normalizeListPayload(payload);
  },
  getOne: async <TData extends BaseRecord = BaseRecord>(params: GetOneParams) => ({
    data: await apiRequest<TData>(`/orgs/${params.id}`),
  }),
  getMany: async <TData extends BaseRecord = BaseRecord>(params: GetManyParams) => {
    const query = new URLSearchParams();
    appendIds(query, params.ids);
    const payload = await apiRequest<ListPayload<TData>>(`/orgs?${query.toString()}`);
    return { data: normalizeListPayload(payload).data };
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
    const ids = resolveFilterIds(params);
    const query = new URLSearchParams();
    const organizationId = resolveOrganizationId(params.meta as MetaLike, params.filters);
    if (organizationId) {
      query.set("organization_id", organizationId);
    }
    const all = resolveBoolean(params.meta as MetaLike, "all");
    if (all) {
      query.set("all", "true");
    }
    appendIds(query, ids);
    if (ids.length === 0) {
      appendPagination(query, params.pagination, 50);
    }
    const path = query.size > 0 ? `/repos?${query.toString()}` : "/repos";
    const payload = await apiRequest<ListPayload<TData>>(path);
    return normalizeListPayload(payload);
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
    const payload = await apiRequest<ListPayload<TData>>(`/repos?${query.toString()}`);
    return { data: normalizeListPayload(payload).data };
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
    const payload = await apiRequest<ListPayload<TData>>(`/orgs/${organizationId}/members`);
    return normalizeListPayload(payload);
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
    const ids = resolveFilterIds(params);
    const query = new URLSearchParams();
    appendIds(query, ids);
    if (ids.length === 0) {
      appendPagination(query, params.pagination, 100);
    }
    const payload = await apiRequest<ListPayload<TData>>(`/users?${query.toString()}`);
    return normalizeListPayload(payload);
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
    const payload = await apiRequest<ListPayload<TData>>(`/users?${query.toString()}`);
    return { data: normalizeListPayload(payload).data };
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
    const target = String(params.id) === "me" ? "/users/me" : `/users/${params.id}`;
    const data = await apiRequest<TData>(target, {
      method: "PATCH",
      body: JSON.stringify(params.variables),
    });
    return { data };
  },
  deleteOne: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(
    params: DeleteOneParams<TVariables>,
  ) => {
    const data = await apiRequest<TData>(`/users/${params.id}`, { method: "DELETE" });
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
