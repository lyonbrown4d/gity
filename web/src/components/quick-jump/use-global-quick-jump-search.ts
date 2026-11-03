import { useEffect, useState } from "react";
import { useCustom, type BaseRecord, type HttpError } from "@refinedev/core";
import type { SearchReply, SearchRequest } from "@/lib/generated/model";

const normalizeQuery = (query: string): string => query.trim();
type RefineSearchReply = SearchReply & BaseRecord;

export const useGlobalQuickJumpSearch = (query: string, enabled: boolean) => {
  const [settledQuery, setSettledQuery] = useState("");
  const normalized = normalizeQuery(query);

  useEffect(() => {
    const timer = window.setTimeout(() => setSettledQuery(normalized), 250);
    return () => window.clearTimeout(timer);
  }, [normalized]);

  const current = enabled && normalized.length >= 1 && normalized === settledQuery;
  const request: SearchRequest = {
    query: settledQuery,
    types: ["code", "issue"],
    limit: 8,
  };
  const search = useCustom<RefineSearchReply, HttpError, unknown, SearchRequest>({
    url: "/search",
    method: "post",
    config: { payload: request },
    queryOptions: {
      enabled: current,
      retry: false,
      gcTime: 0,
      refetchOnMount: "always",
      refetchOnWindowFocus: false,
      staleTime: 0,
    },
  });
  const reply = current && search.query.isSuccess && !search.query.isFetching && !search.query.isError
    ? search.result.data
    : undefined;
  const items = reply?.groups.flatMap((group) => group.items) ?? [];

  return {
    items,
    partial: reply?.partial ?? false,
    hasMore: Boolean(reply?.next_cursor),
    isSearching: enabled && normalized.length >= 1 && (!current || search.query.isFetching),
    isEmpty: current && search.query.isSuccess && items.length === 0,
    error: current && search.query.isError ? search.query.error : null,
    isTooShort: normalized.length < 1,
  };
};
