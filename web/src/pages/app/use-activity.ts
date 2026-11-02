import { useCustom } from "@refinedev/core";
import { toActivityItems, type ActivityItem } from "@/components/activity/activity-feed";

const ACTIVITY_QUERY_KEY = ["activity"] as const;

/**
 * Reads `GET /activity?limit=N`.
 *
 * Orval does not cover this endpoint yet, so the request goes through
 * `useCustom` (the data provider hits `/api/v1/activity`). Switch to the
 * generated `useGetActivity` client after `pnpm gen:api` includes it.
 */
export function useActivity(limit: number) {
  const query = useCustom<Record<string, unknown>>({
    url: "/activity",
    method: "get",
    config: { query: { limit } },
    queryOptions: {
      queryKey: [...ACTIVITY_QUERY_KEY, limit],
      refetchOnWindowFocus: false,
      retry: false,
    },
  });

  const items: ActivityItem[] = toActivityItems(query.result.data);
  return { items, query: query.query };
}
