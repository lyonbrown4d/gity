import type {
  BaseRecord,
  CreateManyParams,
  CreateParams,
  CrudFilters,
  CustomParams,
  DataProvider,
  DeleteManyParams,
  DeleteOneParams,
  GetListParams,
  GetManyParams,
  GetOneParams,
  UpdateManyParams,
  UpdateParams,
} from "@refinedev/core";
import { apiRequest, getApiBaseUrl } from "@/lib/api";

type MetaLike = Record<string, unknown> | undefined;

function asString(value: unknown): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : undefined;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  return value as Record<string, unknown>;
}

function findFilterValue(filters: CrudFilters | undefined, field: string): string | undefined {
  if (!Array.isArray(filters)) {
    return undefined;
  }

  for (const item of filters) {
    const filter = asRecord(item);
    if (!filter) {
      continue;
    }
    if (filter.field === field && filter.operator === "eq") {
      const value = filter.value;
      if (typeof value === "string" || typeof value === "number") {
        return String(value);
      }
    }
  }

  return undefined;
}

function resolveOrganizationId(meta: MetaLike, filters?: CrudFilters): string | undefined {
  const byMetaSnake = asString(meta?.organization_id);
  const byMetaCamel = asString(meta?.organizationId);
  const byFilter = findFilterValue(filters, "organization_id");
  return byMetaSnake ?? byMetaCamel ?? byFilter;
}

function resolveBoolean(meta: MetaLike, key: string): boolean | undefined {
  const value = meta?.[key];
  if (typeof value === "boolean") {
    return value;
  }
  if (typeof value === "string") {
    const normalized = value.trim().toLowerCase();
    if (normalized === "true") {
      return true;
    }
    if (normalized === "false") {
      return false;
    }
  }
  return undefined;
}

function resolveLimit(pagination: unknown, fallback = 100): number {
  const record = asRecord(pagination);
  if (!record) {
    return fallback;
  }

  const pageSize = record.pageSize;
  if (typeof pageSize !== "number" || Number.isNaN(pageSize)) {
    return fallback;
  }

  return Math.max(1, Math.min(200, Math.floor(pageSize)));
}

function buildCustomPath(url: string, query: unknown, filters: unknown, sorters: unknown): string {
  const isAbsolute = url.startsWith("http://") || url.startsWith("https://");
  const base = isAbsolute ? new URL(url) : new URL(url, "http://local.placeholder");

  const queryRecord = asRecord(query);
  if (queryRecord) {
    for (const [key, value] of Object.entries(queryRecord)) {
      if (value === undefined || value === null) {
        continue;
      }
      base.searchParams.set(key, String(value));
    }
  }

  if (Array.isArray(filters)) {
    for (const item of filters) {
      const filter = asRecord(item);
      if (!filter) {
        continue;
      }
      if (typeof filter.field === "string" && filter.operator === "eq") {
        base.searchParams.append(filter.field, String(filter.value));
      }
    }
  }

  if (Array.isArray(sorters)) {
    for (const sorterItem of sorters) {
      const sorter = asRecord(sorterItem);
      if (!sorter) {
        continue;
      }
      const field = asString(sorter.field);
      const order = asString(sorter.order);
      if (field && order) {
        base.searchParams.append(`sort[${field}]`, order);
      }
    }
  }

  if (isAbsolute) {
    return base.toString();
  }
  return `${base.pathname}${base.search}`;
}

function notSupported(resource: string, method: string): never {
  throw new Error(`dataProvider.${method} is not supported for resource "${resource}"`);
}

const getList: DataProvider["getList"] = async <
  TData extends BaseRecord = BaseRecord,
>(params: GetListParams) => {
  const { resource, pagination, filters, meta } = params;
  switch (resource) {
    case "organizations": {
      const data = await apiRequest<TData[]>("/orgs");
      return { data, total: data.length };
    }
    case "repositories":
    case "my-repositories": {
      const organizationId = resolveOrganizationId(meta as MetaLike, filters);
      const query = new URLSearchParams();
      if (organizationId) {
        query.set("organization_id", organizationId);
      }
      const all = resolveBoolean(meta as MetaLike, "all");
      if (all) {
        query.set("all", "true");
      }
      const path = query.toString().length > 0 ? `/repos?${query.toString()}` : "/repos";
      const data = await apiRequest<TData[]>(path);
      return { data, total: data.length };
    }
    case "organization-members": {
      const organizationId = resolveOrganizationId(meta as MetaLike, filters);
      if (!organizationId) {
        throw new Error("organization_id is required for organization-members");
      }
      const data = await apiRequest<TData[]>(`/orgs/${organizationId}/members`);
      return { data, total: data.length };
    }
    case "users": {
      const limit = resolveLimit(pagination, 100);
      const data = await apiRequest<TData[]>(`/users?limit=${limit}`);
      return { data, total: data.length };
    }
    case "profile": {
      const me = await apiRequest<TData>("/users/me");
      return { data: [me], total: 1 };
    }
    default:
      notSupported(resource, "getList");
  }
};

const getOne: DataProvider["getOne"] = async <TData extends BaseRecord = BaseRecord>(
  params: GetOneParams,
) => {
  const { resource, id } = params;
  switch (resource) {
    case "profile":
      return { data: await apiRequest<TData>("/users/me") };
    case "users": {
      if (String(id) === "me") {
        return { data: await apiRequest<TData>("/users/me") };
      }
      const users = await apiRequest<TData[]>("/users?limit=200");
      const found = users.find((item) => String(item.id) === String(id));
      if (!found) {
        throw new Error("user not found");
      }
      return { data: found };
    }
    default:
      notSupported(resource, "getOne");
  }
};

const create: DataProvider["create"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(params: CreateParams<TVariables>) => {
  const { resource, variables } = params;
  switch (resource) {
    case "organizations": {
      const data = await apiRequest<TData>("/orgs", {
        method: "POST",
        body: JSON.stringify(variables),
      });
      return { data };
    }
    case "repositories":
    case "my-repositories": {
      const data = await apiRequest<TData>("/repos", {
        method: "POST",
        body: JSON.stringify(variables),
      });
      return { data };
    }
    case "organization-members": {
      const payload = asRecord(variables as unknown);
      const organizationId =
        asString(payload?.organization_id) ?? asString(payload?.organizationId);
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
    }
    case "users": {
      const data = await apiRequest<TData>("/users", {
        method: "POST",
        body: JSON.stringify(variables),
      });
      return { data };
    }
    default:
      notSupported(resource, "create");
  }
};

const update: DataProvider["update"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(params: UpdateParams<TVariables>) => {
  const { resource, id, variables } = params;
  switch (resource) {
    case "organizations": {
      const data = await apiRequest<TData>(`/orgs/${id}`, {
        method: "PATCH",
        body: JSON.stringify(variables),
      });
      return { data };
    }
    case "profile":
    case "users": {
      const target = String(id) === "me" || resource === "profile" ? "/users/me" : "/users/me";
      const data = await apiRequest<TData>(target, {
        method: "PATCH",
        body: JSON.stringify(variables),
      });
      return { data };
    }
    default:
      notSupported(resource, "update");
  }
};

const deleteOne: DataProvider["deleteOne"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: DeleteOneParams<TVariables>,
) => {
  switch (params.resource) {
    case "organizations": {
      const data = await apiRequest<TData>(`/orgs/${params.id}`, {
        method: "DELETE",
      });
      return { data };
    }
    case "repositories":
    case "my-repositories": {
      const data = await apiRequest<TData>(`/repos/${params.id}`, {
        method: "DELETE",
      });
      return { data };
    }
    default:
      notSupported(params.resource, "deleteOne");
  }
};

const getMany: DataProvider["getMany"] = async <TData extends BaseRecord = BaseRecord>(
  params: GetManyParams,
) => {
  const { resource, ids, meta } = params;
  switch (resource) {
    case "users": {
      const users = await apiRequest<TData[]>("/users?limit=200");
      const idSet = new Set(ids.map((item) => String(item)));
      return { data: users.filter((item: TData) => idSet.has(String(item.id))) };
    }
    case "organizations":
    case "repositories":
    case "my-repositories": {
      const list = await getList<TData>({
        resource,
        pagination: undefined,
        filters: undefined,
        sorters: undefined,
        meta,
      });
      const idSet = new Set(ids.map((item) => String(item)));
      return { data: list.data.filter((item: TData) => idSet.has(String(item.id))) };
    }
    default:
      notSupported(resource, "getMany");
  }
};

const createMany: DataProvider["createMany"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: CreateManyParams<TVariables>,
) => {
  notSupported(params.resource, "createMany");
};

const updateMany: DataProvider["updateMany"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: UpdateManyParams<TVariables>,
) => {
  notSupported(params.resource, "updateMany");
};

const deleteMany: DataProvider["deleteMany"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: DeleteManyParams<TVariables>,
) => {
  notSupported(params.resource, "deleteMany");
};

const custom: DataProvider["custom"] = async <
  TData extends BaseRecord = BaseRecord,
  TQuery = unknown,
  TPayload = unknown,
>(params: CustomParams<TQuery, TPayload>) => {
  const { url, method, filters, sorters, payload, query, headers, meta } = params;
  const path = buildCustomPath(url, query, filters, sorters);
  const requestHeaders = new Headers(headers as HeadersInit | undefined);
  const auth = (meta as Record<string, unknown> | undefined)?.auth as boolean | undefined;
  const data = await apiRequest<TData>(
    path,
    {
      method: method?.toUpperCase() ?? "GET",
      headers: requestHeaders,
      body: payload === undefined ? undefined : JSON.stringify(payload),
    },
    { auth: auth ?? true },
  );
  return { data };
};

export const dataProvider: DataProvider = {
  getApiUrl: () => getApiBaseUrl(),
  getList,
  getOne,
  create,
  update,
  deleteOne,
  getMany,
  createMany,
  updateMany,
  deleteMany,
  custom,
};
