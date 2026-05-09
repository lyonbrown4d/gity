import { useEffect, useMemo, useState } from "react";
import { GitMerge, GitPullRequest, RefreshCw, Search } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type {
  RepositoryBranchView,
  RepositoryMergeRequestDiffView,
  RepositoryMergeRequestState,
  RepositoryMergeRequestView,
} from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";

interface RepositoryMergeRequestsTabProps {
  repoId: string;
  branches: RepositoryBranchView[];
  defaultBranch: string;
  t: (text: string) => string;
  onError: (message: string | null) => void;
  onMerged: () => Promise<void>;
}

type RawRecord = Record<string, unknown>;

export const RepositoryMergeRequestsTab = ({
  repoId,
  branches,
  defaultBranch,
  t,
  onError,
  onMerged,
}: RepositoryMergeRequestsTabProps): JSX.Element => {
  const mergeRequestsQuery = useCustom<RawRecord[]>({
    url: `/repos/${repoId}/merge-requests`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const [selectedIID, setSelectedIID] = useState<number | null>(null);
  const diffQuery = useCustom<RawRecord>({
    url: selectedIID ? `/repos/${repoId}/merge-requests/${selectedIID}/diff` : `/repos/${repoId}/merge-requests/0/diff`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedIID),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: createMergeRequest, isLoading: isCreating } = useCustomMutation<RawRecord>();
  const { mutateAsync: updateMergeRequest, isLoading: isUpdating } = useCustomMutation<RawRecord>();
  const { mutateAsync: mergeMergeRequest, isLoading: isMerging } = useCustomMutation<RawRecord>();
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [stateFilter, setStateFilter] = useState<RepositoryMergeRequestState | "all">("opened");
  const [searchQuery, setSearchQuery] = useState("");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [sourceBranch, setSourceBranch] = useState("");
  const [targetBranch, setTargetBranch] = useState(defaultBranch || "main");

  const mergeRequests = useMemo(
    () => resolveMergeRequestList(mergeRequestsQuery.data?.data).map(normalizeMergeRequest),
    [mergeRequestsQuery.data?.data],
  );
  const selectedMergeRequest = useMemo(
    () => mergeRequests.find((item) => item.iid === selectedIID) ?? mergeRequests[0] ?? null,
    [mergeRequests, selectedIID],
  );
  const filteredMergeRequests = useMemo(
    () => filterMergeRequests(mergeRequests, stateFilter, searchQuery),
    [mergeRequests, stateFilter, searchQuery],
  );
  const stats = useMemo(
    () => ({
      opened: mergeRequests.filter((item) => item.state === "opened").length,
      merged: mergeRequests.filter((item) => item.state === "merged").length,
      closed: mergeRequests.filter((item) => item.state === "closed").length,
    }),
    [mergeRequests],
  );
  const diffView = useMemo(
    () => normalizeDiffView(diffQuery.data?.data),
    [diffQuery.data?.data],
  );
  const isLoadingMergeRequests = mergeRequestsQuery.isFetching && !mergeRequestsQuery.data;

  const loadMergeRequests = async () => {
    const result = await mergeRequestsQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadDiff = async () => {
    if (!selectedIID) {
      return;
    }
    const result = await diffQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const submitCreateMergeRequest = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedTitle = title.trim();
    const normalizedSource = sourceBranch.trim();
    const normalizedTarget = targetBranch.trim();
    if (!normalizedTitle) return onError(t("Merge request title is required"));
    if (!normalizedSource || !normalizedTarget) return onError(t("Source and target branches are required"));
    if (normalizedSource === normalizedTarget) return onError(t("Source and target branches must differ"));

    onError(null);
    try {
      const response = await createMergeRequest({
        url: `/repos/${repoId}/merge-requests`,
        method: "post",
        values: {
          title: normalizedTitle,
          description: description.trim(),
          source_branch: normalizedSource,
          target_branch: normalizedTarget,
        },
      });
      const created = normalizeMergeRequest(response.data);
      setSelectedIID(created.iid);
      setComposerOpen(false);
      setTitle("");
      setDescription("");
      setSourceBranch("");
      setTargetBranch(defaultBranch || "main");
      await loadMergeRequests();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitUpdateState = async (nextState: RepositoryMergeRequestState) => {
    if (!selectedMergeRequest) {
      return;
    }
    onError(null);
    try {
      await updateMergeRequest({
        url: `/repos/${repoId}/merge-requests/${selectedMergeRequest.iid}`,
        method: "patch",
        values: { state: nextState },
      });
      await loadMergeRequests();
      await loadDiff();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitMerge = async () => {
    if (!selectedMergeRequest) {
      return;
    }
    onError(null);
    try {
      await mergeMergeRequest({
        url: `/repos/${repoId}/merge-requests/${selectedMergeRequest.iid}/merge`,
        method: "post",
        values: {},
      });
      await Promise.all([loadMergeRequests(), onMerged()]);
      await loadDiff();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    if (!repoId) {
      return;
    }
    onError(null);
  }, [repoId, onError]);

  useEffect(() => {
    if (selectedIID !== null || mergeRequests.length === 0) {
      return;
    }
    setSelectedIID(mergeRequests[0].iid);
  }, [mergeRequests, selectedIID]);

  useEffect(() => {
    if (!defaultBranch) {
      return;
    }
    setTargetBranch(defaultBranch);
  }, [defaultBranch]);

  useEffect(() => {
    if (!mergeRequestsQuery.error) {
      return;
    }
    onError(extractErrorMessage(mergeRequestsQuery.error));
  }, [mergeRequestsQuery.error, onError]);

  useEffect(() => {
    if (!diffQuery.error) {
      return;
    }
    onError(extractErrorMessage(diffQuery.error));
  }, [diffQuery.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Merge Requests")}</CardTitle>
        <CardDescription>{t("Review branch changes and merge them into protected targets.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <MergeRequestStat label={t("Open")} value={stats.opened} tone="emerald" />
          <MergeRequestStat label={t("Merged")} value={stats.merged} tone="blue" />
          <MergeRequestStat label={t("Closed")} value={stats.closed} tone="slate" />
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(280px,420px)_1fr]">
          <div className="space-y-3">
            <div className="rounded-md border p-3">
              <div className="grid gap-2 md:grid-cols-[1fr_150px] xl:grid-cols-1">
                <div className="relative">
                  <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    className="pl-8"
                    placeholder={t("Search merge requests")}
                    value={searchQuery}
                    onChange={(event) => setSearchQuery(event.target.value)}
                  />
                </div>
                <Select
                  value={stateFilter}
                  onValueChange={(value) => setStateFilter(value as RepositoryMergeRequestState | "all")}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t("Status")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="opened">{t("Open merge requests")}</SelectItem>
                    <SelectItem value="merged">{t("Merged merge requests")}</SelectItem>
                    <SelectItem value="closed">{t("Closed merge requests")}</SelectItem>
                    <SelectItem value="all">{t("All merge requests")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2">
              <Button
                type="button"
                variant={isComposerOpen ? "secondary" : "outline"}
                onClick={() => setComposerOpen((current) => !current)}
              >
                <GitPullRequest className="size-4" />
                {isComposerOpen ? t("Hide new merge request form") : t("New merge request")}
              </Button>
              <Button type="button" size="sm" variant="ghost" onClick={() => void loadMergeRequests()}>
                <RefreshCw className="size-4" />
                {t("Reload")}
              </Button>
            </div>

            {isComposerOpen ? (
              <form className="space-y-3 rounded-md border p-3" onSubmit={submitCreateMergeRequest}>
                <Input
                  placeholder={t("Merge request title")}
                  value={title}
                  onChange={(event) => setTitle(event.target.value)}
                  required
                />
                <textarea
                  className="min-h-24 w-full rounded-md border bg-background px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-ring"
                  placeholder={t("Describe the merge request (optional)")}
                  value={description}
                  onChange={(event) => setDescription(event.target.value)}
                />
                <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-1">
                  <BranchSelect
                    id="source-branch"
                    label={t("Source branch")}
                    value={sourceBranch}
                    branches={branches}
                    placeholder={t("Select source branch")}
                    onChange={setSourceBranch}
                  />
                  <BranchSelect
                    id="target-branch"
                    label={t("Target branch")}
                    value={targetBranch}
                    branches={branches}
                    placeholder={t("Select target branch")}
                    onChange={setTargetBranch}
                  />
                </div>
                <div className="flex justify-end">
                  <Button type="submit" disabled={isCreating}>
                    {isCreating ? t("Creating merge request...") : t("Create merge request")}
                  </Button>
                </div>
              </form>
            ) : null}

            <div className="space-y-2 rounded-md border p-2">
              {isLoadingMergeRequests ? (
                <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading merge requests...")}</p>
              ) : null}
              {!isLoadingMergeRequests && filteredMergeRequests.length === 0 ? (
                <p className="px-2 py-2 text-sm text-muted-foreground">{t("No merge requests found.")}</p>
              ) : null}
              {filteredMergeRequests.map((mergeRequest) => (
                <button
                  key={mergeRequest.id}
                  type="button"
                  className={`w-full rounded-md border p-3 text-left transition hover:bg-muted/40 ${
                    selectedMergeRequest?.iid === mergeRequest.iid ? "border-primary/60 bg-primary/5" : ""
                  }`}
                  onClick={() => setSelectedIID(mergeRequest.iid)}
                >
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <p className="font-medium">
                      !{mergeRequest.iid} {mergeRequest.title}
                    </p>
                    <MergeRequestStateBadge state={mergeRequest.state} t={t} />
                  </div>
                  <p className="mt-2 text-xs text-muted-foreground">
                    {mergeRequest.source_branch} → {mergeRequest.target_branch}
                    {mergeRequest.updated_at ? ` · ${formatRelativeTime(mergeRequest.updated_at)}` : ""}
                  </p>
                </button>
              ))}
            </div>
          </div>

          <div className="min-w-0 rounded-md border p-3">
            {!selectedMergeRequest ? (
              <p className="text-sm text-muted-foreground">{t("Select a merge request to inspect the diff.")}</p>
            ) : (
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0 space-y-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <h3 className="text-lg font-semibold">
                        !{selectedMergeRequest.iid} {selectedMergeRequest.title}
                      </h3>
                      <MergeRequestStateBadge state={selectedMergeRequest.state} t={t} />
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {selectedMergeRequest.source_branch} → {selectedMergeRequest.target_branch}
                    </p>
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    <Button type="button" size="sm" variant="ghost" onClick={() => void loadDiff()}>
                      <RefreshCw className="size-4" />
                      {t("Reload diff")}
                    </Button>
                    {selectedMergeRequest.state === "opened" ? (
                      <>
                        <Button type="button" size="sm" variant="outline" disabled={isUpdating} onClick={() => void submitUpdateState("closed")}>
                          {t("Close merge request")}
                        </Button>
                        <Button type="button" size="sm" disabled={isMerging} onClick={() => void submitMerge()}>
                          <GitMerge className="size-4" />
                          {isMerging ? t("Merging...") : t("Merge")}
                        </Button>
                      </>
                    ) : null}
                    {selectedMergeRequest.state === "closed" ? (
                      <Button type="button" size="sm" variant="outline" disabled={isUpdating} onClick={() => void submitUpdateState("opened")}>
                        {t("Reopen merge request")}
                      </Button>
                    ) : null}
                  </div>
                </div>

                {selectedMergeRequest.description ? (
                  <p className="rounded-md border bg-muted/20 px-3 py-2 text-sm">{selectedMergeRequest.description}</p>
                ) : null}

                <div>
                  <p className="mb-2 text-sm font-medium">{t("Diff")}</p>
                  {diffQuery.isFetching ? (
                    <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("Loading diff...")}</p>
                  ) : (
                    <pre className="max-h-[620px] overflow-auto rounded-md border bg-zinc-950 p-3 text-xs leading-relaxed text-zinc-100">
                      {diffView?.diff?.trim() ? diffView.diff : t("No diff available.")}
                    </pre>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const BranchSelect = ({
  id,
  label,
  value,
  branches,
  placeholder,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  branches: RepositoryBranchView[];
  placeholder: string;
  onChange: (value: string) => void;
}) => (
  <div className="space-y-1">
    <Label className="text-xs font-medium text-muted-foreground" htmlFor={id}>
      {label}
    </Label>
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger id={id}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        {branches.map((branch) => (
          <SelectItem key={branch.name} value={branch.name}>
            {branch.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  </div>
);

const MergeRequestStat = ({ label, value, tone }: { label: string; value: number; tone: "emerald" | "blue" | "slate" }) => {
  const toneClass = {
    emerald: "border-emerald-500/30 bg-emerald-500/5",
    blue: "border-blue-500/30 bg-blue-500/5",
    slate: "border-slate-500/30 bg-slate-500/5",
  }[tone];
  return (
    <div className={`rounded-md border p-3 ${toneClass}`}>
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="text-lg font-semibold">{value}</p>
    </div>
  );
};

const MergeRequestStateBadge = ({ state, t }: { state: RepositoryMergeRequestState; t: (text: string) => string }) => {
  const variant = state === "opened" ? "default" : state === "merged" ? "secondary" : "outline";
  const label = state === "opened" ? "Open" : state === "merged" ? "Merged" : "Closed";
  return <Badge variant={variant}>{t(label)}</Badge>;
};

const filterMergeRequests = (
  items: RepositoryMergeRequestView[],
  state: RepositoryMergeRequestState | "all",
  query: string,
): RepositoryMergeRequestView[] => {
  const normalizedQuery = query.trim().toLowerCase();
  return items
    .filter((item) => state === "all" || item.state === state)
    .filter((item) => {
      if (!normalizedQuery) {
        return true;
      }
      return `${item.iid} ${item.title} ${item.description ?? ""} ${item.source_branch} ${item.target_branch}`
        .toLowerCase()
        .includes(normalizedQuery);
    })
    .sort((a, b) => {
      const left = Date.parse(a.updated_at ?? "");
      const right = Date.parse(b.updated_at ?? "");
      return (Number.isFinite(right) ? right : 0) - (Number.isFinite(left) ? left : 0);
    });
};

const resolveMergeRequestList = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  if (!isRecord(payload)) {
    return [];
  }
  const nested = payload.body ?? payload.Body;
  if (Array.isArray(nested)) {
    return nested.filter(isRecord);
  }
  return [];
};

const normalizeDiffView = (payload: unknown): RepositoryMergeRequestDiffView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    diff: normalizeString(raw.diff ?? raw.Diff),
  };
};

const normalizeMergeRequest = (rawValue: unknown): RepositoryMergeRequestView => {
  const raw = isRecord(rawValue) ? rawValue : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    iid: normalizeNumber(raw.iid ?? raw.IID),
    author_user_id: normalizeString(raw.author_user_id ?? raw.AuthorUserID),
    title: normalizeString(raw.title ?? raw.Title),
    description: normalizeOptionalString(raw.description ?? raw.Description),
    state: normalizeState(raw.state ?? raw.State),
    source_branch: normalizeString(raw.source_branch ?? raw.SourceBranch),
    target_branch: normalizeString(raw.target_branch ?? raw.TargetBranch),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const isRecord = (value: unknown): value is RawRecord =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const normalizeString = (value: unknown): string => {
  if (value === undefined || value === null) {
    return "";
  }
  return String(value);
};

const normalizeOptionalString = (value: unknown): string | null => {
  const normalized = normalizeString(value).trim();
  if (!normalized || normalized === "0001-01-01T00:00:00Z") {
    return null;
  }
  return normalized;
};

const normalizeNumber = (value: unknown): number => {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  const parsed = Number.parseInt(normalizeString(value), 10);
  return Number.isFinite(parsed) ? parsed : 0;
};

const normalizeState = (value: unknown): RepositoryMergeRequestState => {
  const normalized = normalizeString(value);
  if (normalized === "closed" || normalized === "merged") {
    return normalized;
  }
  return "opened";
};
