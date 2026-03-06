import type {
  BaseRecord,
  CreateManyParams,
  CreateParams,
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
import { resolveAdapter } from "./refine/resource-adapters";
import { buildCustomPath, filterByIds, notSupported, requireValue } from "./refine/shared";

const getList: DataProvider["getList"] = async <TData extends BaseRecord = BaseRecord>(
  params: GetListParams,
) => {
  const handler = requireValue(resolveAdapter(params.resource)?.getList, params.resource, "getList");
  return handler<TData>(params);
};

const getOne: DataProvider["getOne"] = async <TData extends BaseRecord = BaseRecord>(
  params: GetOneParams,
) => {
  const handler = requireValue(resolveAdapter(params.resource)?.getOne, params.resource, "getOne");
  return handler<TData>(params);
};

const create: DataProvider["create"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: CreateParams<TVariables>,
) => {
  const handler = requireValue(resolveAdapter(params.resource)?.create, params.resource, "create");
  return handler<TData, TVariables>(params);
};

const update: DataProvider["update"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: UpdateParams<TVariables>,
) => {
  const handler = requireValue(resolveAdapter(params.resource)?.update, params.resource, "update");
  return handler<TData, TVariables>(params);
};

const deleteOne: DataProvider["deleteOne"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: DeleteOneParams<TVariables>,
) => {
  const handler = requireValue(resolveAdapter(params.resource)?.deleteOne, params.resource, "deleteOne");
  return handler<TData, TVariables>(params);
};

const getMany: DataProvider["getMany"] = async <TData extends BaseRecord = BaseRecord>(
  params: GetManyParams,
) => {
  const adapter = requireValue(resolveAdapter(params.resource), params.resource, "getMany");
  if (adapter.getMany) {
    return adapter.getMany<TData>(params);
  }

  const getListHandler = requireValue(adapter.getList, params.resource, "getMany");
  const list = await getListHandler<TData>({
    resource: params.resource,
    pagination: undefined,
    filters: undefined,
    sorters: undefined,
    meta: params.meta,
  });

  return { data: filterByIds(list.data, params.ids.map((item) => String(item))) };
};

const createMany: DataProvider["createMany"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: CreateManyParams<TVariables>,
) => notSupported(params.resource, "createMany");

const updateMany: DataProvider["updateMany"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: UpdateManyParams<TVariables>,
) => notSupported(params.resource, "updateMany");

const deleteMany: DataProvider["deleteMany"] = async <
  TData extends BaseRecord = BaseRecord,
  TVariables = {},
>(
  params: DeleteManyParams<TVariables>,
) => notSupported(params.resource, "deleteMany");

const custom: DataProvider["custom"] = async <
  TData extends BaseRecord = BaseRecord,
  TQuery = unknown,
  TPayload = unknown,
>(
  params: CustomParams<TQuery, TPayload>,
) => {
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
