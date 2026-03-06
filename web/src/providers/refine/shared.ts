import type { BaseRecord, CrudFilters, DataProvider } from "@refinedev/core";

export type MetaLike = Record<string, unknown> | undefined;

export type ResourceAdapter = {
  getList?: DataProvider["getList"];
  getOne?: DataProvider["getOne"];
  create?: DataProvider["create"];
  update?: DataProvider["update"];
  deleteOne?: DataProvider["deleteOne"];
  getMany?: DataProvider["getMany"];
};

export const asString = (value: unknown): string | undefined => {
  if (typeof value !== "string") {
    return undefined;
  }
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : undefined;
};

export const asRecord = (value: unknown): Record<string, unknown> | null => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  return value as Record<string, unknown>;
};

const findFilterValue = (filters: CrudFilters | undefined, field: string): string | undefined => {
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
};

export const resolveOrganizationId = (meta: MetaLike, filters?: CrudFilters): string | undefined => {
  const byMetaSnake = asString(meta?.organization_id);
  const byMetaCamel = asString(meta?.organizationId);
  const byFilter = findFilterValue(filters, "organization_id");
  return byMetaSnake ?? byMetaCamel ?? byFilter;
};

export const resolveBoolean = (meta: MetaLike, key: string): boolean | undefined => {
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
};

export const resolveLimit = (pagination: unknown, fallback = 100): number => {
  const record = asRecord(pagination);
  if (!record) {
    return fallback;
  }

  const pageSize = record.pageSize;
  if (typeof pageSize !== "number" || Number.isNaN(pageSize)) {
    return fallback;
  }

  return Math.max(1, Math.min(200, Math.floor(pageSize)));
};

export const notSupported = (resource: string, method: string): never => {
  throw new Error(`dataProvider.${method} is not supported for resource "${resource}"`);
};

export const requireValue = <T>(value: T | undefined, resource: string, method: string): T => {
  if (value === undefined) {
    notSupported(resource, method);
  }
  return value as T;
};

export const filterByIds = <TData extends BaseRecord = BaseRecord>(
  items: TData[],
  ids: Array<string | number>,
): TData[] => {
  const idSet = new Set(ids.map((item) => String(item)));
  return items.filter((item) => idSet.has(String(item.id)));
};

export const buildCustomPath = (
  url: string,
  query: unknown,
  filters: unknown,
  sorters: unknown,
): string => {
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
};
