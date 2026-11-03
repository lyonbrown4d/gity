import type {
  BaseRecord,
  CreateParams,
  DeleteOneParams,
  GetListParams,
  GetManyParams,
  GetOneParams,
  UpdateParams,
} from "@refinedev/core";
import { clientList, validateListFilters } from "./list-behavior";
import { listUsers, usersByIds } from "./user-list";
import { generatedRequest } from "./generated-dispatch";
import {
  asRecord,
  asString,
  filterByIds,
  normalizeListPayload,
  type PagePayload,
  resolveOrganizationId,
  type MetaLike,
  type ResourceAdapter,
} from "./shared";

type ListPayload<TData extends BaseRecord = BaseRecord> = TData[] | PagePayload<TData>;

const organizationsAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    validateListFilters(params);
    const payload = await generatedRequest<TData[]>("/orgs");
    return clientList(payload, params);
  },
  getOne: async <TData extends BaseRecord = BaseRecord>(params: GetOneParams) => ({
    data: await generatedRequest<TData>(`/orgs/${params.id}`),
  }),
  getMany: async <TData extends BaseRecord = BaseRecord>(params: GetManyParams) => {
    // The backend does not filter by `ids`; fetch the visible list and filter locally.
    const payload = await generatedRequest<ListPayload<TData>>("/orgs");
    return { data: filterByIds(normalizeListPayload(payload).data, params.ids) };
  },
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: CreateParams<TVariables>) => {
    const data = await generatedRequest<TData>("/orgs", {
      method: "POST",
      payload: params.variables,
    });
    return { data };
  },
  update: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: UpdateParams<TVariables>) => {
    const data = await generatedRequest<TData>(`/orgs/${params.id}`, {
      method: "PATCH",
      payload: params.variables,
    });
    return { data };
  },
  deleteOne: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: DeleteOneParams<TVariables>) => {
    const data = await generatedRequest<TData>(`/orgs/${params.id}`, { method: "DELETE" });
    return { data };
  },
};

const projectsAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    validateListFilters(params, ["organization_id"]);
    if ((params.meta as MetaLike)?.all !== undefined)
      throw new Error("The projects endpoint does not support the all filter");
    const query = new URLSearchParams();
    const organizationId = resolveOrganizationId(params.meta as MetaLike, params.filters);
    if (organizationId) query.set("organization_id", organizationId);
    const payload = await generatedRequest<TData[]>(query.size ? `/projects?${query}` : "/projects");
    return clientList(payload, params);
  },
  getOne: async <TData extends BaseRecord = BaseRecord>(params: GetOneParams) => ({
    data: await generatedRequest<TData>(`/projects/${params.id}`),
  }),
  getMany: async <TData extends BaseRecord = BaseRecord>(params: GetManyParams) => {
    const query = new URLSearchParams();
    const organizationId = resolveOrganizationId(params.meta as MetaLike);
    if (organizationId) {
      query.set("organization_id", organizationId);
    }
    if ((params.meta as MetaLike)?.all !== undefined)
      throw new Error("The projects endpoint does not support the all filter");
    // The backend does not filter by `ids`; fetch the visible list and filter locally.
    const payload = await generatedRequest<ListPayload<TData>>(query.size ? `/projects?${query}` : "/projects");
    return { data: filterByIds(normalizeListPayload(payload).data, params.ids) };
  },
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: CreateParams<TVariables>) => {
    const data = await generatedRequest<TData>("/projects", {
      method: "POST",
      payload: params.variables,
    });
    return { data };
  },
  deleteOne: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: DeleteOneParams<TVariables>) => {
    const meta = asRecord(params.meta as unknown);
    const confirmation = asString(meta?.confirmation);
    const data = await generatedRequest<TData>(`/projects/${params.id}`, {
      method: "DELETE",
      payload: confirmation ? { confirmation } : {},
    });
    return { data };
  },
};

const organizationMembersAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    validateListFilters(params, ["organization_id"]);
    const organizationId = resolveOrganizationId(params.meta as MetaLike, params.filters);
    if (!organizationId) {
      throw new Error("organization_id is required for organization-members");
    }
    const payload = await generatedRequest<ListPayload<TData>>(`/orgs/${organizationId}/members`);
    return clientList(normalizeListPayload(payload).data, params);
  },
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: CreateParams<TVariables>) => {
    const payload = asRecord(params.variables as unknown);
    const organizationId = asString(payload?.organization_id) ?? asString(payload?.organizationId);
    if (!organizationId) {
      throw new Error("organization_id is required for organization-members");
    }

    const data = await generatedRequest<TData>(`/orgs/${organizationId}/members`, {
      method: "POST",
      payload: {
        user_id: payload?.user_id ?? payload?.userId,
        role: payload?.role,
      },
    });
    return { data };
  },
};

const usersAdapter: ResourceAdapter = {
  getList: listUsers,
  getOne: async <TData extends BaseRecord = BaseRecord>(params: GetOneParams) => {
    if (String(params.id) === "me") {
      return { data: await generatedRequest<TData>("/users/me") };
    }
    return { data: await generatedRequest<TData>(`/users/${params.id}`) };
  },
  getMany: async <TData extends BaseRecord = BaseRecord>(params: GetManyParams) => ({
    data: await usersByIds<TData>(params.ids),
  }),
  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: CreateParams<TVariables>) => {
    const data = await generatedRequest<TData>("/users", {
      method: "POST",
      payload: params.variables,
    });
    return { data };
  },
  update: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: UpdateParams<TVariables>) => {
    const target = String(params.id) === "me" ? "/users/me" : `/users/${params.id}`;
    const data = await generatedRequest<TData>(target, {
      method: "PATCH",
      payload: params.variables,
    });
    return { data };
  },
  deleteOne: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: DeleteOneParams<TVariables>) => {
    const data = await generatedRequest<TData>(`/users/${params.id}`, { method: "DELETE" });
    return { data };
  },
};

const profileAdapter: ResourceAdapter = {
  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    validateListFilters(params);
    const me = await generatedRequest<TData>("/users/me");
    return clientList([me], params);
  },
  getOne: async <TData extends BaseRecord = BaseRecord>() => ({
    data: await generatedRequest<TData>("/users/me"),
  }),
  update: async <TData extends BaseRecord = BaseRecord, TVariables = {}>(params: UpdateParams<TVariables>) => {
    const data = await generatedRequest<TData>("/users/me", {
      method: "PATCH",
      payload: params.variables,
    });
    return { data };
  },
};

const resourceAdapters: Record<string, ResourceAdapter> = {
  organizations: organizationsAdapter,
  projects: projectsAdapter,
  "my-projects": projectsAdapter,
  repositories: projectsAdapter,
  "my-repositories": projectsAdapter,
  "organization-members": organizationMembersAdapter,
  users: usersAdapter,
  profile: profileAdapter,
};

export const resolveAdapter = (resource: string): ResourceAdapter | undefined => resourceAdapters[resource];
