import { useEffect, useMemo, useState } from "react";
import { Activity, ArrowRight, BookOpen, Code2, Copy, GitBranch, GitCommit, GitPullRequest, ListTodo, Package, PlayCircle, Rocket, ShieldCheck, Tag, Users, type LucideIcon } from "lucide-react";
import { useCustom } from "@refinedev/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { RepositoryBlobView, RepositoryBranchView, RepositoryCommitView, RepositoryView } from "@/pages/types";
import { formatRelativeTime, toTimestamp } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeBoolean, normalizeNumber, normalizeString, resolveArrayPayload, resolveBody, type RawRecord } from "./repository-normalizers";
import type { RepoTab } from "./repository-types";
import { renderMarkdown, shortSha } from "./repository-utils";

interface RepositoryOverviewTabProps {
  repoId: string;
  repository: RepositoryView;
  branches: RepositoryBranchView[];
  commits: RepositoryCommitView[];
  isLoadingCommits: boolean;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onOpenTab: (tab: RepoTab) => void;
  onCopyCloneUrl: () => void;
}

interface InfoTileProps {
  label: string;
  value: string;
  description?: string;
  icon: LucideIcon;
}

interface ActionSummaryCardProps {
  label: string;
  value: number;
  description: string;
  actionLabel: string;
  icon: LucideIcon;
  tab: RepoTab;
  isLoading: boolean;
  onOpenTab: (tab: RepoTab) => void;
}

const activeWorkStatuses = new Set(["pending", "running"]);
const failedStatuses = new Set(["failed"]);
const recentCommitLimit = 5;

export const RepositoryOverviewTab = ({
  repoId,
  repository,
  branches,
  commits,
  isLoadingCommits,
  permissions,
  t,
  onOpenTab,
  onCopyCloneUrl,
}: RepositoryOverviewTabProps): JSX.Element => {
  const readmeBranch = repository.default_branch || "main";
  const readmeQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/repository/readme`,
    method: "get",
    config: { query: { branch_name: readmeBranch } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const issuesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/issues`,
    method: "get",
    config: { query: { status: "all", limit: 100 } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const mergeRequestsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/merge-requests`,
    method: "get",
    config: { query: { limit: 100 } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const pipelinesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/pipelines`,
    method: "get",
    config: { query: { limit: 100 } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const jobsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/jobs`,
    method: "get",
    config: { query: { limit: 100 } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const runnersQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/runners`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const wikiQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/wiki/pages`,
    method: "get",
    config: { query: { limit: 100 } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const packagesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/packages`,
    method: "get",
    config: { query: { limit: 100 } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const releasesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/releases`,
    method: "get",
    config: { query: { limit: 100 } },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });

  const readmeBlob = useMemo(
    () => normalizeReadmeBlob(readmeQuery.result.data),
    [readmeQuery.result.data],
  );
  const issues = useMemo(() => resolveRecords(issuesQuery.result.data), [issuesQuery.result.data]);
  const mergeRequests = useMemo(() => resolveRecords(mergeRequestsQuery.result.data), [mergeRequestsQuery.result.data]);
  const pipelines = useMemo(() => resolveRecords(pipelinesQuery.result.data), [pipelinesQuery.result.data]);
  const jobs = useMemo(() => resolveRecords(jobsQuery.result.data), [jobsQuery.result.data]);
  const runners = useMemo(() => resolveRecords(runnersQuery.result.data), [runnersQuery.result.data]);
  const wikiPages = useMemo(() => resolveRecords(wikiQuery.result.data), [wikiQuery.result.data]);
  const packages = useMemo(() => resolveRecords(packagesQuery.result.data), [packagesQuery.result.data]);
  const releases = useMemo(() => resolveRecords(releasesQuery.result.data), [releasesQuery.result.data]);
  const [readmeHtml, setReadmeHtml] = useState<string | null>(null);

  useEffect(() => {
    if (!readmeBlob || readmeBlob.is_binary) {
      setReadmeHtml(null);
      return;
    }

    let cancelled = false;
    void renderMarkdown(readmeBlob.content).then((html) => {
      if (!cancelled) {
        setReadmeHtml(html);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [readmeBlob?.content, readmeBlob?.is_binary, readmeBlob?.path]);

  const recentCommits = useMemo(() => {
    const sorted = [...commits].sort((a, b) => toTimestamp(b.created_at) - toTimestamp(a.created_at));
    return sorted.slice(0, recentCommitLimit);
  }, [commits]);
  const latestCommit = recentCommits[0] ?? null;
  const protectedBranches = branches.filter((branch) => branch.is_protected).length;
  const stats = useMemo(
    () => ({
      openIssues: countMatchingString(issues, ["status", "Status"], "open"),
      openMergeRequests: countMatchingString(mergeRequests, ["state", "State"], "opened"),
      activePipelines: countMatchingSet(pipelines, ["status", "Status"], activeWorkStatuses),
      failedPipelines: countMatchingSet(pipelines, ["status", "Status"], failedStatuses),
      activeJobs: countMatchingSet(jobs, ["status", "Status"], activeWorkStatuses),
      failedJobs: countMatchingSet(jobs, ["status", "Status"], failedStatuses),
      onlineRunners: countMatchingString(runners, ["status", "Status"], "online"),
      inactiveRunners: runners.filter((item) => hasBooleanValue(item, ["active", "Active"]) && !getBoolean(item, ["active", "Active"])).length,
      wikiPages: wikiPages.length,
      packages: packages.length,
      releases: releases.length,
    }),
    [issues, jobs, mergeRequests, packages, pipelines, releases, runners, wikiPages],
  );
  const actionCards = useMemo(
    () => [
      {
        label: t("Open issues"),
        value: stats.openIssues,
        description: stats.openIssues > 0 ? t("Needs triage or assignment") : t("Issue backlog is clear"),
        actionLabel: stats.openIssues > 0 ? t("Review issues") : t("Create issue"),
        icon: ListTodo,
        tab: "issues" as const,
        isLoading: isQueryLoading(issuesQuery),
      },
      {
        label: t("Open merge requests"),
        value: stats.openMergeRequests,
        description: stats.openMergeRequests > 0 ? t("Waiting for review or merge") : t("No merge requests open"),
        actionLabel: stats.openMergeRequests > 0 ? t("Review MRs") : t("Open MRs"),
        icon: GitPullRequest,
        tab: "merge-requests" as const,
        isLoading: isQueryLoading(mergeRequestsQuery),
      },
      {
        label: t("Active pipelines"),
        value: stats.activePipelines,
        description: stats.failedPipelines > 0 ? `${stats.failedPipelines} ${t("failed pipeline(s)")}` : t("CI pipeline queue is calm"),
        actionLabel: stats.activePipelines > 0 ? t("Inspect pipelines") : t("Run pipeline"),
        icon: Rocket,
        tab: "pipelines" as const,
        isLoading: isQueryLoading(pipelinesQuery),
      },
      {
        label: t("Active jobs"),
        value: stats.activeJobs,
        description: stats.failedJobs > 0 ? `${stats.failedJobs} ${t("failed job(s)")}` : t("No blocked background work"),
        actionLabel: stats.activeJobs > 0 ? t("Inspect jobs") : t("Open jobs"),
        icon: PlayCircle,
        tab: "jobs" as const,
        isLoading: isQueryLoading(jobsQuery),
      },
      {
        label: t("Online runners"),
        value: stats.onlineRunners,
        description: stats.inactiveRunners > 0 ? `${stats.inactiveRunners} ${t("inactive runner(s)")}` : t("Runner fleet is available"),
        actionLabel: stats.onlineRunners > 0 ? t("Manage runners") : t("Register runner"),
        icon: Users,
        tab: "runners" as const,
        isLoading: isQueryLoading(runnersQuery),
      },
      {
        label: t("Wiki pages"),
        value: stats.wikiPages,
        description: stats.wikiPages > 0 ? t("Project knowledge base exists") : t("Create onboarding docs"),
        actionLabel: stats.wikiPages > 0 ? t("Open wiki") : t("Create wiki"),
        icon: BookOpen,
        tab: "wiki" as const,
        isLoading: isQueryLoading(wikiQuery),
      },
      {
        label: t("Packages"),
        value: stats.packages,
        description: stats.packages > 0 ? t("Artifacts are published") : t("Publish build artifacts"),
        actionLabel: stats.packages > 0 ? t("View packages") : t("Publish package"),
        icon: Package,
        tab: "packages" as const,
        isLoading: isQueryLoading(packagesQuery),
      },
      {
        label: t("Releases"),
        value: stats.releases,
        description: stats.releases > 0 ? t("Tagged deliverables are available") : t("Cut the first release"),
        actionLabel: stats.releases > 0 ? t("View releases") : t("Draft release"),
        icon: Tag,
        tab: "releases" as const,
        isLoading: isQueryLoading(releasesQuery),
      },
    ],
    [
      issuesQuery,
      jobsQuery,
      mergeRequestsQuery,
      packagesQuery,
      pipelinesQuery,
      releasesQuery,
      runnersQuery,
      stats,
      t,
      wikiQuery,
    ],
  );

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(340px,0.85fr)]">
      <div className="flex flex-col gap-4">
        <Card className="card-enter">
          <CardHeader>
            <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
              <div>
                <CardTitle>{t("Project overview")}</CardTitle>
                <CardDescription>{t("Repository home summary, clone details, and current project activity.")}</CardDescription>
              </div>
              <Button type="button" variant="outline" size="sm" onClick={() => onOpenTab("code")}>
                <Code2 data-icon="inline-start" />
                {t("Browse code")}
              </Button>
            </div>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <div className="rounded-xl border border-border/80 bg-background/70 p-3">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div className="min-w-0">
                  <p className="text-xs font-medium text-muted-foreground">{t("Clone URL")}</p>
                  <p className="break-all font-mono text-xs text-foreground">{repository.clone_http_url}</p>
                </div>
                <Button type="button" size="sm" className="shrink-0" onClick={onCopyCloneUrl}>
                  <Copy data-icon="inline-start" />
                  {t("Copy")}
                </Button>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              <InfoTile icon={GitBranch} label={t("Default branch")} value={repository.default_branch || "main"} description={`${protectedBranches} ${t("protected")}`} />
              <InfoTile icon={Activity} label={t("Visibility")} value={repository.visibility || t("N/A")} description={repository.status ?? t("N/A")} />
              <InfoTile icon={ShieldCheck} label={t("Access role")} value={t(permissions.roleLabel)} description={permissions.canWrite ? t("Write access") : t("Read-only access")} />
              <InfoTile icon={GitCommit} label={t("Latest commit")} value={latestCommit ? shortSha(latestCommit.commit_sha) : t("None")} description={latestCommit ? formatRelativeTime(latestCommit.created_at) : t("No commits yet")} />
            </div>
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
              <div>
                <CardTitle>{t("README")}</CardTitle>
                <CardDescription>{readmeBlob?.path ?? t("Quick entry to project documentation.")}</CardDescription>
              </div>
              <Button type="button" variant="outline" size="sm" onClick={() => onOpenTab("code")}>
                <BookOpen data-icon="inline-start" />
                {readmeBlob ? t("View README") : t("Open code")}
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {isQueryLoading(readmeQuery) ? (
              <div className="flex flex-col gap-3">
                <Skeleton className="h-5 w-2/3" />
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-5/6" />
                <Skeleton className="h-24 w-full" />
              </div>
            ) : readmeBlob?.is_binary ? (
              <p className="text-sm text-muted-foreground">{t("README is binary and cannot be rendered here. Open the code tab to inspect it.")}</p>
            ) : readmeHtml ? (
              <article className="markdown-body max-h-[34rem] overflow-auto" dangerouslySetInnerHTML={{ __html: readmeHtml }} />
            ) : (
              <div className="flex flex-col gap-3 rounded-xl border border-border/80 bg-background/70 p-4">
                <p className="text-sm font-medium">{t("No README found")}</p>
                <p className="text-sm text-muted-foreground">{t("Add a README to turn this project home into a useful landing page for contributors.")}</p>
                <div>
                  <Button type="button" variant="outline" size="sm" onClick={() => onOpenTab("code")}>
                    {t("Open repository files")}
                    <ArrowRight data-icon="inline-end" />
                  </Button>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="flex flex-col gap-4">
        <Card className="card-enter">
          <CardHeader>
            <div className="flex items-start justify-between gap-3">
              <div>
                <CardTitle>{t("Recent commits")}</CardTitle>
                <CardDescription>{t("Latest activity across repository branches.")}</CardDescription>
              </div>
              <Button type="button" variant="outline" size="sm" onClick={() => onOpenTab("commits")}>
                {t("View all")}
              </Button>
            </div>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            {isLoadingCommits ? (
              <div className="flex flex-col gap-3">
                <Skeleton className="h-16 w-full" />
                <Skeleton className="h-16 w-full" />
                <Skeleton className="h-16 w-full" />
              </div>
            ) : recentCommits.length > 0 ? (
              recentCommits.map((commit) => (
                <div key={`${commit.branch_name}-${commit.commit_sha}`} className="rounded-xl border border-border/80 bg-background/70 p-3">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline">{shortSha(commit.commit_sha)}</Badge>
                    <Badge variant="secondary">{commit.branch_name}</Badge>
                  </div>
                  <p className="mt-2 line-clamp-2 text-sm font-medium">{firstLine(commit.message) || t("No commit message")}</p>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {commit.author_user_id} · {formatRelativeTime(commit.created_at)}
                  </p>
                </div>
              ))
            ) : (
              <p className="text-sm text-muted-foreground">{t("No commits found.")}</p>
            )}
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("Next actions")}</CardTitle>
            <CardDescription>{t("Jump into the project areas that need attention next.")}</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3 sm:grid-cols-2 xl:grid-cols-1 2xl:grid-cols-2">
            {actionCards.map((card) => (
              <ActionSummaryCard
                key={card.tab}
                label={card.label}
                value={card.value}
                description={card.description}
                actionLabel={card.actionLabel}
                icon={card.icon}
                tab={card.tab}
                isLoading={card.isLoading}
                onOpenTab={onOpenTab}
              />
            ))}
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

const InfoTile = ({ label, value, description, icon: Icon }: InfoTileProps): JSX.Element => (
  <div className="rounded-xl border border-border/80 bg-background/70 p-3">
    <div className="flex items-center justify-between gap-3">
      <p className="text-xs text-muted-foreground">{label}</p>
      <Icon className="size-4 text-muted-foreground" />
    </div>
    <p className="mt-2 truncate text-lg font-semibold">{value}</p>
    {description ? <p className="mt-1 truncate text-xs text-muted-foreground">{description}</p> : null}
  </div>
);

const ActionSummaryCard = ({
  label,
  value,
  description,
  actionLabel,
  icon: Icon,
  tab,
  isLoading,
  onOpenTab,
}: ActionSummaryCardProps): JSX.Element => (
  <Card className="bg-background/70 shadow-sm">
    <CardHeader className="flex-row items-start justify-between gap-3 pb-3">
      <div className="min-w-0">
        <CardTitle className="truncate text-sm">{label}</CardTitle>
        <CardDescription className="mt-1 line-clamp-2">{description}</CardDescription>
      </div>
      <div className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-border/80 bg-card">
        <Icon className="size-4 text-muted-foreground" />
      </div>
    </CardHeader>
    <CardContent className="flex items-end justify-between gap-3">
      {isLoading ? <Skeleton className="h-8 w-14" /> : <p className="text-3xl font-semibold tabular-nums">{value}</p>}
      <Button type="button" variant="outline" size="sm" onClick={() => onOpenTab(tab)}>
        {actionLabel}
      </Button>
    </CardContent>
  </Card>
);

const isQueryLoading = (query: { query: { isFetching: boolean; data?: unknown } }): boolean =>
  query.query.isFetching && query.query.data === undefined;

const resolveRecords = (payload: unknown): RawRecord[] => resolveArrayPayload<unknown>(payload).filter(isRecord);

const normalizeReadmeBlob = (payload: unknown): RepositoryBlobView | null => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return null;
  }
  return {
    path: pickString(raw, ["path", "Path"]),
    content: pickString(raw, ["content", "Content"]),
    size: normalizeNumber(raw.size ?? raw.Size),
    is_binary: normalizeBoolean(raw.is_binary ?? raw.IsBinary),
    encoding: pickString(raw, ["encoding", "Encoding"]),
  };
};

const pickString = (raw: RawRecord, keys: string[]): string => {
  for (const key of keys) {
    const value = normalizeString(raw[key]).trim();
    if (value) {
      return value;
    }
  }
  return "";
};

const getBoolean = (raw: RawRecord, keys: string[]): boolean => {
  for (const key of keys) {
    if (raw[key] !== undefined && raw[key] !== null) {
      return normalizeBoolean(raw[key]);
    }
  }
  return false;
};

const hasBooleanValue = (raw: RawRecord, keys: string[]): boolean =>
  keys.some((key) => raw[key] !== undefined && raw[key] !== null);

const countMatchingString = (items: RawRecord[], keys: string[], expected: string): number =>
  items.filter((item) => pickString(item, keys).toLowerCase() === expected).length;

const countMatchingSet = (items: RawRecord[], keys: string[], expected: Set<string>): number =>
  items.filter((item) => expected.has(pickString(item, keys).toLowerCase())).length;

const firstLine = (value: string): string => value.split(/\r?\n/g)[0]?.trim() ?? "";
