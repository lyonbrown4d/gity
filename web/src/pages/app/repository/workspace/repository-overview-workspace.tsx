import { useEffect, useMemo, useState } from "react";
import { BookOpen, GitPullRequest, ListTodo, Package, PlayCircle, Rocket, Tag, Users } from "lucide-react";
import { useCustom } from "@refinedev/core";
import type { RepositoryBlobView, RepositoryBranchView, RepositoryCommitView, RepositoryView } from "@/pages/types";
import { toTimestamp } from "../issues-utils";
import {
  isRecord, normalizeBoolean, normalizeNumber, normalizeString,
  resolveArrayPayload, resolveBody, type RawRecord,
} from "../repository-normalizers";
import type { RepositoryPermissions } from "../repository-permissions";
import type { RepoTab } from "../repository-types";
import { renderMarkdown } from "../repository-utils";
import { RepositoryOverviewLayout, type RepositoryPulseItem } from "./repository-overview-layout";

interface RepositoryOverviewWorkspaceProps {
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

const activeWorkStatuses = new Set(["pending", "running"]);
const failedStatuses = new Set(["failed"]);

export const RepositoryOverviewWorkspace = ({
  repoId, repository, branches, commits, isLoadingCommits, permissions,
  t, onOpenTab, onCopyCloneUrl,
}: RepositoryOverviewWorkspaceProps): JSX.Element => {
  const queryOptions = {
    enabled: Boolean(repoId),
    refetchOnWindowFocus: false as const,
    retry: false as const,
  };
  const listConfig = { query: { limit: 100 } };
  const readmeQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/repository/readme`,
    method: "get",
    config: { query: { branch_name: repository.default_branch || "main" } },
    queryOptions,
  });
  const issuesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/issues`,
    method: "get",
    config: { query: { status: "all", limit: 100 } },
    queryOptions,
  });
  const mergeRequestsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/merge-requests`,
    method: "get",
    config: listConfig,
    queryOptions,
  });
  const pipelinesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/pipelines`,
    method: "get",
    config: listConfig,
    queryOptions,
  });
  const jobsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/jobs`,
    method: "get",
    config: listConfig,
    queryOptions,
  });
  const runnersQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/runners`,
    method: "get",
    queryOptions,
  });
  const wikiQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/wiki/pages`,
    method: "get",
    config: listConfig,
    queryOptions,
  });
  const packagesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/packages`,
    method: "get",
    config: listConfig,
    queryOptions,
  });
  const releasesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/releases`,
    method: "get",
    config: listConfig,
    queryOptions,
  });

  const readmeBlob = useMemo(() => normalizeReadmeBlob(readmeQuery.result.data), [readmeQuery.result.data]);
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
      if (!cancelled) setReadmeHtml(html);
    });
    return () => {
      cancelled = true;
    };
  }, [readmeBlob?.content, readmeBlob?.is_binary, readmeBlob?.path]);

  const recentCommits = useMemo(
    () => [...commits].sort((a, b) => toTimestamp(b.created_at) - toTimestamp(a.created_at)).slice(0, 5),
    [commits],
  );
  const stats = {
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
  };
  const pulseItems: RepositoryPulseItem[] = [
    {
      label: t("Open issues"), value: stats.openIssues,
      description: stats.openIssues > 0 ? t("Needs triage or assignment") : t("Issue backlog is clear"),
      actionLabel: stats.openIssues > 0 ? t("Review issues") : t("Create issue"),
      icon: ListTodo, tab: "issues", isLoading: isQueryLoading(issuesQuery),
    },
    {
      label: t("Open merge requests"), value: stats.openMergeRequests,
      description: stats.openMergeRequests > 0 ? t("Waiting for review or merge") : t("No merge requests open"),
      actionLabel: stats.openMergeRequests > 0 ? t("Review MRs") : t("Open MRs"),
      icon: GitPullRequest, tab: "merge-requests", isLoading: isQueryLoading(mergeRequestsQuery),
    },
    {
      label: t("Active pipelines"), value: stats.activePipelines,
      description: stats.failedPipelines > 0 ? String(stats.failedPipelines) + " " + t("failed pipeline(s)") : t("CI pipeline queue is calm"),
      actionLabel: stats.activePipelines > 0 ? t("Inspect pipelines") : t("Run pipeline"),
      icon: Rocket, tab: "pipelines", isLoading: isQueryLoading(pipelinesQuery),
    },
    {
      label: t("Active jobs"), value: stats.activeJobs,
      description: stats.failedJobs > 0 ? String(stats.failedJobs) + " " + t("failed job(s)") : t("No blocked background work"),
      actionLabel: stats.activeJobs > 0 ? t("Inspect jobs") : t("Open jobs"),
      icon: PlayCircle, tab: "jobs", isLoading: isQueryLoading(jobsQuery),
    },
    {
      label: t("Online runners"), value: stats.onlineRunners,
      description: stats.inactiveRunners > 0 ? String(stats.inactiveRunners) + " " + t("inactive runner(s)") : t("Runner fleet is available"),
      actionLabel: stats.onlineRunners > 0 ? t("Manage runners") : t("Register runner"),
      icon: Users, tab: "runners", isLoading: isQueryLoading(runnersQuery),
    },
    {
      label: t("Wiki pages"), value: stats.wikiPages,
      description: stats.wikiPages > 0 ? t("Project knowledge base exists") : t("Create onboarding docs"),
      actionLabel: stats.wikiPages > 0 ? t("Open wiki") : t("Create wiki"),
      icon: BookOpen, tab: "wiki", isLoading: isQueryLoading(wikiQuery),
    },
    {
      label: t("Packages"), value: stats.packages,
      description: stats.packages > 0 ? t("Artifacts are published") : t("Publish build artifacts"),
      actionLabel: stats.packages > 0 ? t("View packages") : t("Publish package"),
      icon: Package, tab: "packages", isLoading: isQueryLoading(packagesQuery),
    },
    {
      label: t("Releases"), value: stats.releases,
      description: stats.releases > 0 ? t("Tagged deliverables are available") : t("Cut the first release"),
      actionLabel: stats.releases > 0 ? t("View releases") : t("Draft release"),
      icon: Tag, tab: "releases", isLoading: isQueryLoading(releasesQuery),
    },
  ];

  return (
    <RepositoryOverviewLayout
      repository={repository}
      permissions={permissions}
      readmeBlob={readmeBlob}
      readmeHtml={readmeHtml}
      isLoadingReadme={isQueryLoading(readmeQuery)}
      recentCommits={recentCommits}
      latestCommit={recentCommits[0] ?? null}
      isLoadingCommits={isLoadingCommits}
      protectedBranches={branches.filter((branch) => branch.is_protected).length}
      pulseItems={pulseItems}
      t={t}
      onOpenTab={onOpenTab}
      onCopyCloneUrl={onCopyCloneUrl}
    />
  );
};

const isQueryLoading = (query: { query: { isFetching: boolean; data?: unknown } }): boolean =>
  query.query.isFetching && query.query.data === undefined;

const resolveRecords = (payload: unknown): RawRecord[] => resolveArrayPayload<unknown>(payload).filter(isRecord);

const normalizeReadmeBlob = (payload: unknown): RepositoryBlobView | null => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) return null;
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
    if (value) return value;
  }
  return "";
};

const getBoolean = (raw: RawRecord, keys: string[]): boolean => {
  for (const key of keys) {
    if (raw[key] !== undefined && raw[key] !== null) return normalizeBoolean(raw[key]);
  }
  return false;
};

const hasBooleanValue = (raw: RawRecord, keys: string[]): boolean =>
  keys.some((key) => raw[key] !== undefined && raw[key] !== null);

const countMatchingString = (items: RawRecord[], keys: string[], expected: string): number =>
  items.filter((item) => pickString(item, keys).toLowerCase() === expected).length;

const countMatchingSet = (items: RawRecord[], keys: string[], expected: Set<string>): number =>
  items.filter((item) => expected.has(pickString(item, keys).toLowerCase())).length;
