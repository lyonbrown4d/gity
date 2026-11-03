import type { BaseRecord, GetListParams } from "@refinedev/core";
import { resolvePagination } from "./shared";
export const listIds = (params: GetListParams): string[] | undefined => {
  const filter = params.filters?.find((item) => "field" in item && item.field === "id" && item.operator === "in");
  if (!filter || !("value" in filter)) return undefined;
  if (!Array.isArray(filter.value) || filter.value.some((id) => typeof id !== "string" && typeof id !== "number"))
    throw new Error("The id filter requires an array of identifiers");
  return filter.value.map(String);
};
export const validateListFilters = (params: GetListParams, remoteFields: string[] = []): void => {
  const seen = new Map<string, string>();
  for (const filter of params.filters ?? []) {
    if ("field" in filter) {
      const value = JSON.stringify(filter.value);
      if (seen.has(filter.field) && seen.get(filter.field) !== value)
        throw new Error(`Conflicting list filters for ${filter.field}`);
      seen.set(filter.field, value);
    }
    if (
      !("field" in filter) ||
      !(
        (filter.field === "id" && filter.operator === "in") ||
        (remoteFields.includes(filter.field) && filter.operator === "eq")
      )
    ) {
      throw new Error(`Unsupported list filter for ${params.resource}`);
    }
  }
  for (const filter of params.filters ?? []) {
    if (
      "field" in filter &&
      remoteFields.includes(filter.field) &&
      ((typeof filter.value !== "string" && typeof filter.value !== "number") ||
        !String(filter.value).trim() ||
        (typeof filter.value === "number" && !Number.isFinite(filter.value)))
    )
      throw new Error(`Invalid list filter for ${filter.field}`);
  }
  listIds(params);
};
/** The collection endpoints have no server pagination; adapt Refine pagination locally. */
export const clientList = <T extends BaseRecord>(items: T[], params: GetListParams): { data: T[]; total: number } => {
  if (!Array.isArray(items)) throw new Error("Expected a collection API response");
  const ids = listIds(params);
  let data = ids === undefined ? items : items.filter((item) => ids.includes(String(item.id)));
  for (const sorter of params.sorters ?? []) {
    if (
      data.some(
        (item) => !(sorter.field in item) || !["string", "number", "boolean"].includes(typeof item[sorter.field]),
      )
    )
      throw new Error(`Unsupported list sort field: ${sorter.field}`);
  }
  if (params.sorters?.length)
    data = [...data].sort((left, right) => {
      for (const sorter of params.sorters ?? []) {
        const a: unknown = left[sorter.field];
        const b: unknown = right[sorter.field];
        const order = typeof a === "number" && typeof b === "number" ? a - b : String(a).localeCompare(String(b));
        if (order) return sorter.order === "desc" ? -order : order;
      }
      return 0;
    });
  const total = data.length;
  if (!params.pagination || params.pagination.mode === "off" || params.pagination.mode === "client")
    return { data, total };
  const { page, pageSize } = resolvePagination(params.pagination);
  return { data: data.slice((page - 1) * pageSize, page * pageSize), total };
};
