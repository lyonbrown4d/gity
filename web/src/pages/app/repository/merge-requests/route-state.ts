import type { MergeRequestBranchFilter, MergeRequestStateFilter } from "./filters";

export interface MergeRequestRouteFilters {
  state: MergeRequestStateFilter;
  query: string;
  source: MergeRequestBranchFilter;
  target: MergeRequestBranchFilter;
}

const validStates: readonly MergeRequestStateFilter[] = ["opened", "merged", "closed", "all"];

export const parseMergeRequestIID = (value: string | undefined): number | null => {
  if (!value || !/^\d+$/.test(value)) return null;
  const iid = Number(value);
  return Number.isSafeInteger(iid) && iid > 0 ? iid : null;
};

export const readMergeRequestFilters = (params: URLSearchParams): MergeRequestRouteFilters => {
  const requestedState = params.get("state") as MergeRequestStateFilter | null;
  const source = params.get("source") ?? "all";
  const target = params.get("target") ?? "all";
  return {
    state: requestedState && validStates.includes(requestedState) ? requestedState : "opened",
    query: params.get("q") ?? "",
    source: source === "all" || source.startsWith("branch:") ? source as MergeRequestBranchFilter : "all",
    target: target === "all" || target.startsWith("branch:") ? target as MergeRequestBranchFilter : "all",
  };
};

export const updateMergeRequestFilters = (
  current: URLSearchParams,
  patch: Partial<MergeRequestRouteFilters>,
): URLSearchParams => {
  const next = new URLSearchParams(current);
  const values: Record<keyof MergeRequestRouteFilters, string> = {
    state: patch.state ?? readMergeRequestFilters(current).state,
    query: patch.query ?? readMergeRequestFilters(current).query,
    source: patch.source ?? readMergeRequestFilters(current).source,
    target: patch.target ?? readMergeRequestFilters(current).target,
  };
  const defaults: Record<keyof MergeRequestRouteFilters, string> = {
    state: "opened", query: "", source: "all", target: "all",
  };
  const names: Record<keyof MergeRequestRouteFilters, string> = {
    state: "state", query: "q", source: "source", target: "target",
  };
  for (const key of Object.keys(values) as (keyof MergeRequestRouteFilters)[]) {
    if (values[key] === defaults[key]) next.delete(names[key]);
    else next.set(names[key], values[key]);
  }
  return next;
};

export const mergeRequestRoute = (organizationID: string, repoID: string, iid?: number, search = ""): string => {
  const base = `/app/projects/${encodeURIComponent(organizationID)}/${encodeURIComponent(repoID)}/merge-requests`;
  return `${base}${iid ? `/${iid}` : ""}${search}`;
};
