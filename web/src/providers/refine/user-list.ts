import type { BaseRecord, GetListParams } from "@refinedev/core";
import { generatedRequest } from "./generated-dispatch";
import { clientList, listIds, validateListFilters } from "./list-behavior";
import { normalizeListPayload, resolvePagination, type PagePayload } from "./shared";
export const usersByIds = async <T extends BaseRecord>(ids: (string | number)[]): Promise<T[]> => {
  const data: T[] = [];
  // No batch-by-id endpoint exists; bound the concurrent point lookups.
  for (let index = 0; index < ids.length; index += 20)
    data.push(
      ...(await Promise.all(
        ids.slice(index, index + 20).map((id) => generatedRequest<T>(`/users/${encodeURIComponent(String(id))}`)),
      )),
    );
  return data;
};
export const listUsers = async <T extends BaseRecord>(params: GetListParams): Promise<{ data: T[]; total: number }> => {
  validateListFilters(params);
  const ids = listIds(params);
  if (ids !== undefined) return clientList(await usersByIds<T>(ids), params);
  if (params.pagination?.mode === "off" || params.pagination?.mode === "client") {
    const first = await generatedRequest<PagePayload<T>>("/users?page=1&page_size=100");
    const data = [...first.items];
    const pages = Math.ceil(first.total / 100);
    for (let start = 2; start <= pages; start += 5) {
      const results = await Promise.all(
        Array.from({ length: Math.min(5, pages - start + 1) }, (_, index) =>
          generatedRequest<PagePayload<T>>(`/users?page=${start + index}&page_size=100`),
        ),
      );
      data.push(...results.flatMap((page) => page.items));
    }
    return clientList(data, params);
  }
  if (params.sorters?.length) throw new Error("The users endpoint does not support server sorting");
  const { page, pageSize } = resolvePagination(params.pagination, 1, 100, 100);
  return normalizeListPayload(await generatedRequest<PagePayload<T>>(`/users?page=${page}&page_size=${pageSize}`));
};
