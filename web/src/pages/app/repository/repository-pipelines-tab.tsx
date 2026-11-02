import { useEffect, useMemo, useState } from "react";
import { Ban, CheckCircle2, Clock3, Download, FileArchive, GitBranch, Loader2, Play, RefreshCw, Repeat2, ScrollText, XCircle } from "lucide-react";
import { useCustom, useCustomMutation, useDataProvider } from "@refinedev/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";
import type {
  RepositoryJobArtifactContentView,
  RepositoryJobArtifactView,
  RepositoryJobStatus,
  RepositoryJobTraceView,
  RepositoryJobView,
  RepositoryPipelineDetailView,
  RepositoryPipelineJobStatus,
  RepositoryPipelineJobView,
  RepositoryPipelineStatus,
  RepositoryPipelineView,
} from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import {
  isRecord,
  normalizeBoolean,
  normalizeNumber,
  normalizeOptionalString,
  normalizeString,
  normalizeStringArray,
  resolveBody,
  resolveRecordArray,
  type RawRecord,
} from "./repository-normalizers";

interface RepositoryPipelinesTabProps {
  repoId: string;
  defaultBranch: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

const terminalPipelineStatuses: RepositoryPipelineStatus[] = ["succeeded", "failed", "cancelled"];
const terminalJobStatuses: RepositoryJobStatus[] = ["succeeded", "failed", "cancelled"];
const emptyPipelineJobs: RepositoryPipelineJobView[] = [];
const defaultPipelineConfig = `pipeline {
  name = "manual"
}

stage test {
  run {
    shell("go test ./...")
  }
}
`;

export const RepositoryPipelinesTab = ({ repoId, defaultBranch, permissions, t, onError }: RepositoryPipelinesTabProps): JSX.Element => {
  const dataProvider = useDataProvider();
  const pipelinesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/pipelines`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const [selectedPipelineID, setSelectedPipelineID] = useState<string | null>(null);
  const [selectedJobID, setSelectedJobID] = useState<string | null>(null);
  const detailQuery = useCustom<RawRecord>({
    url: selectedPipelineID ? `/projects/${repoId}/pipelines/${selectedPipelineID}` : `/projects/${repoId}/pipelines/0`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedPipelineID),
      refetchOnWindowFocus: false,
    },
  });
  const traceQuery = useCustom<RawRecord>({
    url: selectedJobID ? `/projects/${repoId}/jobs/${selectedJobID}/trace` : `/projects/${repoId}/jobs/0/trace`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedJobID),
      refetchOnWindowFocus: false,
    },
  });
  const artifactsQuery = useCustom<RawRecord[]>({
    url: selectedJobID ? `/projects/${repoId}/jobs/${selectedJobID}/artifacts` : `/projects/${repoId}/jobs/0/artifacts`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedJobID),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: cancelPipeline, mutation: { isPending: isCancelling } } = useCustomMutation<RawRecord>();
  const { mutateAsync: retryPipeline, mutation: { isPending: isRetrying } } = useCustomMutation<RawRecord>();
  const { mutateAsync: refreshPipeline, mutation: { isPending: isRefreshingPipeline } } = useCustomMutation<RawRecord>();
  const { mutateAsync: createPipeline, mutation: { isPending: isCreatingPipeline } } = useCustomMutation<RawRecord>();
  const { mutateAsync: lintPipeline, mutation: { isPending: isLintingPipeline } } = useCustomMutation<RawRecord>();
  const { mutateAsync: cancelJob, mutation: { isPending: isCancellingJob } } = useCustomMutation<RawRecord>();
  const { mutateAsync: retryJob, mutation: { isPending: isRetryingJob } } = useCustomMutation<RawRecord>();
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [pipelineSource, setPipelineSource] = useState("web");
  const [pipelineRefName, setPipelineRefName] = useState(defaultBranch || "main");
  const [pipelineCommitSHA, setPipelineCommitSHA] = useState("");
  const [pipelineConfigSource, setPipelineConfigSource] = useState(".gity-ci.plano");
  const [pipelineConfigContent, setPipelineConfigContent] = useState(defaultPipelineConfig);
  const [lintResult, setLintResult] = useState<string | null>(null);
  const canMutateCI = permissions.ciWrite;
  const canMutateJobs = permissions.jobWrite;

  const pipelines = useMemo(
    () => resolvePipelineList(pipelinesQuery.result.data).map(normalizePipeline).sort((a, b) => b.iid - a.iid),
    [pipelinesQuery.result.data],
  );
  const selectedPipeline = useMemo(
    () => pipelines.find((item) => item.id === selectedPipelineID) ?? pipelines[0] ?? null,
    [pipelines, selectedPipelineID],
  );
  const detail = useMemo(
    () => normalizePipelineDetail(detailQuery.result.data),
    [detailQuery.result.data],
  );
  const visibleDetail = detail?.pipeline.id === selectedPipelineID ? detail : null;
  const pipelineJobs = visibleDetail?.jobs ?? emptyPipelineJobs;
  const visiblePipeline = visibleDetail?.pipeline ?? selectedPipeline;
  const selectedJob = useMemo(
    () => pipelineJobs.find((item) => item.project_job.id === selectedJobID) ?? null,
    [pipelineJobs, selectedJobID],
  );
  const trace = useMemo(
    () => normalizeJobTrace(traceQuery.result.data),
    [traceQuery.result.data],
  );
  const artifacts = useMemo(
    () => resolveArtifactList(artifactsQuery.result.data).map(normalizeArtifact),
    [artifactsQuery.result.data],
  );
  const stats = useMemo(
    () => ({
      pending: pipelines.filter((item) => item.status === "pending").length,
      running: pipelines.filter((item) => item.status === "running").length,
      succeeded: pipelines.filter((item) => item.status === "succeeded").length,
      failed: pipelines.filter((item) => item.status === "failed").length,
    }),
    [pipelines],
  );
  const pipelineActivitySummary = useMemo(
    () => getPipelineActivitySummary(pipelines, t),
    [pipelines, t],
  );
  const isLoadingPipelines = pipelinesQuery.query.isFetching && !pipelinesQuery.query.data;

  const loadPipelines = async () => {
    const result = await pipelinesQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadDetail = async () => {
    if (!selectedPipelineID) {
      return;
    }
    const result = await detailQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadJobDetail = async () => {
    if (!selectedJobID) {
      return;
    }
    const [traceResult, artifactsResult] = await Promise.all([traceQuery.query.refetch(), artifactsQuery.query.refetch()]);
    if (traceResult.error) {
      onError(extractErrorMessage(traceResult.error));
      return;
    }
    if (artifactsResult.error) {
      onError(extractErrorMessage(artifactsResult.error));
      return;
    }
    onError(null);
  };

  const submitLintPipeline = async () => {
    const normalizedConfig = pipelineConfigContent.trim();
    if (!normalizedConfig) {
      onError(t("CI config content is required"));
      return;
    }
    onError(null);
    try {
      const response = await lintPipeline({
        url: `/projects/${repoId}/ci/lint`,
        method: "post",
        values: { config_content: normalizedConfig },
      });
      setLintResult(JSON.stringify(resolveBody(response.data), null, 2));
    } catch (error) {
      setLintResult(null);
      onError(extractErrorMessage(error));
    }
  };

  const submitCreatePipeline = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedRef = pipelineRefName.trim();
    const normalizedConfig = pipelineConfigContent.trim();
    if (!normalizedRef) {
      onError(t("Pipeline ref is required"));
      return;
    }
    if (!normalizedConfig) {
      onError(t("CI config content is required"));
      return;
    }

    onError(null);
    try {
      const response = await createPipeline({
        url: `/projects/${repoId}/pipelines`,
        method: "post",
        values: {
          source: pipelineSource,
          ref_name: normalizedRef,
          commit_sha: pipelineCommitSHA.trim(),
          config_source: pipelineConfigSource.trim() || ".gity-ci.plano",
          config_content: normalizedConfig,
        },
      });
      const created = normalizePipelineDetail(response.data);
      if (created?.pipeline.id) {
        setSelectedPipelineID(created.pipeline.id);
      }
      setLintResult(null);
      setComposerOpen(false);
      await loadPipelines();
      await loadDetail();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitRefreshPipeline = async () => {
    if (!visiblePipeline) {
      return;
    }
    onError(null);
    try {
      const response = await refreshPipeline({
        url: `/projects/${repoId}/pipelines/${visiblePipeline.id}/refresh`,
        method: "post",
        values: {},
      });
      const refreshed = normalizePipelineDetail(response.data);
      if (refreshed?.pipeline.id) {
        setSelectedPipelineID(refreshed.pipeline.id);
      }
      await Promise.all([loadPipelines(), loadDetail()]);
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitCancelPipeline = async () => {
    if (!visiblePipeline) {
      return;
    }
    onError(null);
    try {
      await cancelPipeline({
        url: `/projects/${repoId}/pipelines/${visiblePipeline.id}/cancel`,
        method: "post",
        values: {},
      });
      await Promise.all([loadPipelines(), loadDetail()]);
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitRetryPipeline = async () => {
    if (!visiblePipeline) {
      return;
    }
    onError(null);
    try {
      await retryPipeline({
        url: `/projects/${repoId}/pipelines/${visiblePipeline.id}/retry`,
        method: "post",
        values: {},
      });
      await Promise.all([loadPipelines(), loadDetail()]);
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitCancelJob = async (job: RepositoryPipelineJobView) => {
    onError(null);
    try {
      await cancelJob({
        url: `/projects/${repoId}/jobs/${job.project_job.id}/cancel`,
        method: "post",
        values: {},
      });
      await Promise.all([loadPipelines(), loadDetail(), loadJobDetail()]);
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitRetryJob = async (job: RepositoryPipelineJobView) => {
    onError(null);
    try {
      await retryJob({
        url: `/projects/${repoId}/jobs/${job.project_job.id}/retry`,
        method: "post",
        values: {},
      });
      await Promise.all([loadPipelines(), loadDetail(), loadJobDetail()]);
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const downloadArtifact = async (artifact: RepositoryJobArtifactView) => {
    if (!selectedJob) {
      return;
    }
    const custom = dataProvider().custom;
    if (!custom) {
      onError(t("Artifact download is not available."));
      return;
    }
    onError(null);
    try {
      const response = await custom<RawRecord>({
        url: `/projects/${repoId}/jobs/${selectedJob.project_job.id}/artifacts/${artifact.id}`,
        method: "get",
      });
      const content = normalizeArtifactContent(response.data);
      const bytes = decodeBase64(content.content_base64);
      const blob = new Blob([bytes], { type: content.artifact.content_type || "application/octet-stream" });
      const objectURL = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = objectURL;
      link.download = content.artifact.file_name || artifact.file_name || "artifact";
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(objectURL);
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
    setPipelineRefName((current) => current.trim() || defaultBranch || "main");
  }, [defaultBranch]);

  useEffect(() => {
    if (selectedPipelineID !== null || pipelines.length === 0) {
      return;
    }
    setSelectedPipelineID(pipelines[0].id);
  }, [pipelines, selectedPipelineID]);

  useEffect(() => {
    setSelectedJobID(null);
  }, [selectedPipelineID]);

  useEffect(() => {
    if (pipelineJobs.length === 0) {
      setSelectedJobID(null);
      return;
    }
    if (selectedJobID && pipelineJobs.some((item) => item.project_job.id === selectedJobID)) {
      return;
    }
    setSelectedJobID(pipelineJobs[0].project_job.id);
  }, [pipelineJobs, selectedJobID]);

  useEffect(() => {
    if (!pipelinesQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(pipelinesQuery.query.error));
  }, [pipelinesQuery.query.error, onError]);

  useEffect(() => {
    if (!detailQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(detailQuery.query.error));
  }, [detailQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Pipelines")}</CardTitle>
        <CardDescription>{t("Track CI pipelines triggered by pushes and merge requests.")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-4">
          <PipelineStat label={t("Pending")} value={stats.pending} description={stats.pending > 0 ? t("Waiting for execution") : t("No queued pipelines")} status="pending" />
          <PipelineStat label={t("Running")} value={stats.running} description={stats.running > 0 ? t("Jobs are executing") : t("No active pipelines")} status="running" />
          <PipelineStat label={t("Succeeded")} value={stats.succeeded} description={stats.succeeded > 0 ? t("Healthy completions") : t("No successes yet")} status="succeeded" />
          <PipelineStat label={t("Failed")} value={stats.failed} description={stats.failed > 0 ? t("Needs investigation") : t("No failures")} status="failed" />
        </div>

        <Alert className="bg-muted/20">
          <AlertTitle>{t("Pipeline status summary")}</AlertTitle>
          <AlertDescription className="flex flex-col gap-2">
            <p>{pipelineActivitySummary}</p>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline" className={stats.failed > 0 ? "border-destructive/30 bg-destructive/10 text-destructive" : undefined}>{t("Failures")}: {stats.failed}</Badge>
              <Badge variant={stats.running > 0 ? "secondary" : "outline"}>{t("Running")}: {stats.running}</Badge>
              <Badge variant={stats.pending > 0 ? "secondary" : "outline"}>{t("Queued")}: {stats.pending}</Badge>
            </div>
          </AlertDescription>
        </Alert>

        {!canMutateCI ? (
          <Alert>
            <AlertDescription>
              {t("Your current project role can inspect CI, but cannot create or mutate pipelines.")}
            </AlertDescription>
          </Alert>
        ) : null}

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isComposerOpen ? "secondary" : "outline"}
            disabled={!canMutateCI}
            onClick={() => setComposerOpen((current) => !current)}
          >
            <Play data-icon="inline-start" />
            {isComposerOpen ? t("Hide pipeline form") : t("New pipeline")}
          </Button>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="secondary">
              {t("Role")}: {t(permissions.roleLabel)}
            </Badge>
            <Button type="button" size="sm" variant="ghost" onClick={() => void loadPipelines()}>
              <RefreshCw data-icon="inline-start" />
              {t("Reload")}
            </Button>
          </div>
        </div>

        {isComposerOpen ? (
          <form className="flex flex-col gap-3 rounded-md border bg-muted/10 p-3" onSubmit={submitCreatePipeline}>
            <div className="grid gap-3 md:grid-cols-[160px_1fr_1fr]">
              <div className="flex flex-col gap-1">
                <Label htmlFor="pipeline-source" className="text-xs text-muted-foreground">
                  {t("Source")}
                </Label>
                <Select value={pipelineSource} onValueChange={setPipelineSource}>
                  <SelectTrigger id="pipeline-source">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="web">{t("web")}</SelectItem>
                    <SelectItem value="manual">{t("manual")}</SelectItem>
                    <SelectItem value="push">{t("push")}</SelectItem>
                    <SelectItem value="merge_request">{t("merge_request")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="pipeline-ref" className="text-xs text-muted-foreground">
                  {t("Ref")}
                </Label>
                <Input
                  id="pipeline-ref"
                  value={pipelineRefName}
                  onChange={(event) => setPipelineRefName(event.target.value)}
                  placeholder={defaultBranch || "main"}
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="pipeline-commit" className="text-xs text-muted-foreground">
                  {t("Commit SHA")}
                </Label>
                <Input
                  id="pipeline-commit"
                  value={pipelineCommitSHA}
                  onChange={(event) => setPipelineCommitSHA(event.target.value)}
                  placeholder={t("Optional")}
                />
              </div>
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="pipeline-config-source" className="text-xs text-muted-foreground">
                {t("Config source")}
              </Label>
              <Input
                id="pipeline-config-source"
                value={pipelineConfigSource}
                onChange={(event) => setPipelineConfigSource(event.target.value)}
                placeholder=".gity-ci.plano"
              />
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="pipeline-config-content" className="text-xs text-muted-foreground">
                {t("Plano CI config")}
              </Label>
              <Textarea
                id="pipeline-config-content"
                className="min-h-64 font-mono text-xs"
                value={pipelineConfigContent}
                onChange={(event) => setPipelineConfigContent(event.target.value)}
              />
            </div>
            {lintResult ? (
              <pre className="max-h-72 overflow-auto rounded-md bg-background p-3 text-xs">{lintResult}</pre>
            ) : null}
            <div className="flex flex-wrap justify-end gap-2">
              <Button
                type="button"
                variant="outline"
                disabled={!canMutateCI || isLintingPipeline}
                onClick={() => void submitLintPipeline()}
              >
                {isLintingPipeline ? t("Linting...") : t("Lint config")}
              </Button>
              <Button type="submit" disabled={!canMutateCI || isCreatingPipeline}>
                {isCreatingPipeline ? t("Creating pipeline...") : t("Create pipeline")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="grid gap-4 xl:grid-cols-[minmax(280px,420px)_1fr]">
          <div className="flex flex-col gap-3">
            <div className="flex items-center justify-between gap-2">
              <p className="text-sm font-medium">{t("Pipelines")}</p>
            </div>
            <div className="flex flex-col gap-2 rounded-md border p-2">
              {isLoadingPipelines ? <PipelineListSkeleton /> : null}
              {!isLoadingPipelines && pipelines.length === 0 ? (
                <PipelineEmptyState canMutate={canMutateCI} t={t} onCreate={() => setComposerOpen(true)} />
              ) : null}
              {pipelines.map((pipeline) => (
                <button
                  key={pipeline.id}
                  type="button"
                  className={cn(
                    "w-full rounded-md border bg-background/60 p-3 text-left transition hover:bg-muted/40",
                    visiblePipeline?.id === pipeline.id ? "border-primary/60 bg-primary/5" : undefined,
                    pipeline.status === "failed" ? "border-destructive/30 bg-destructive/5" : undefined,
                  )}
                  aria-pressed={visiblePipeline?.id === pipeline.id}
                  onClick={() => setSelectedPipelineID(pipeline.id)}
                >
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <p className="font-medium">
                      #{pipeline.iid} {pipeline.name || "pipeline"}
                    </p>
                    <PipelineStatusBadge status={pipeline.status} t={t} />
                  </div>
                  <p className="mt-2 flex flex-wrap items-center gap-1 text-xs text-muted-foreground">
                    <GitBranch className="size-3" />
                    <span>{pipeline.ref_name || t("N/A")}</span>
                    <span>·</span>
                    <span>{shortSHA(pipeline.commit_sha)}</span>
                    {pipeline.updated_at ? <span>· {formatRelativeTime(pipeline.updated_at)}</span> : null}
                  </p>
                </button>
              ))}
            </div>
          </div>

          <div className="min-w-0 rounded-md border p-3">
            {!visiblePipeline ? (
              <PipelinePanelEmptyState t={t} />
            ) : (
              <div className="flex flex-col gap-4">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0 flex flex-col gap-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <h3 className="text-lg font-semibold">
                        #{visiblePipeline.iid} {visiblePipeline.name || "pipeline"}
                      </h3>
                      <PipelineStatusBadge status={visiblePipeline.status} t={t} />
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {t("Source")}: {visiblePipeline.source || t("N/A")} · {t("Ref")}: {visiblePipeline.ref_name || t("N/A")} · {t("Commit")}: {shortSHA(visiblePipeline.commit_sha)}
                    </p>
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    <Button type="button" size="sm" variant="ghost" onClick={() => void loadDetail()}>
                      <RefreshCw data-icon="inline-start" />
                      {t("Reload pipeline")}
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={!canMutateCI || isRefreshingPipeline}
                      onClick={() => void submitRefreshPipeline()}
                    >
                      <RefreshCw data-icon="inline-start" />
                      {isRefreshingPipeline ? t("Refreshing...") : t("Refresh state")}
                    </Button>
                    {!terminalPipelineStatuses.includes(visiblePipeline.status) ? (
                      <Button type="button" size="sm" variant="outline" disabled={!canMutateCI || isCancelling} onClick={() => void submitCancelPipeline()}>
                        <Ban data-icon="inline-start" />
                        {isCancelling ? t("Cancelling...") : t("Cancel pipeline")}
                      </Button>
                    ) : null}
                    {visiblePipeline.status !== "running" ? (
                      <Button type="button" size="sm" variant="outline" disabled={!canMutateCI || isRetrying} onClick={() => void submitRetryPipeline()}>
                        <Repeat2 data-icon="inline-start" />
                        {isRetrying ? t("Retrying...") : t("Retry pipeline")}
                      </Button>
                    ) : null}
                  </div>
                </div>

                <div className="grid gap-3 md:grid-cols-3">
                  <PipelineMeta label={t("Config")} value={visiblePipeline.config_source || t("N/A")} />
                  <PipelineMeta label={t("Started")} value={visiblePipeline.started_at ? formatRelativeTime(visiblePipeline.started_at) : t("N/A")} />
                  <PipelineMeta label={t("Finished")} value={visiblePipeline.finished_at ? formatRelativeTime(visiblePipeline.finished_at) : t("N/A")} />
                </div>

                <PipelineDiagnosticsSummary pipeline={visiblePipeline} jobs={pipelineJobs} canMutate={canMutateCI} t={t} />

                <div className="flex flex-col gap-2">
                  <p className="text-sm font-medium">{t("Pipeline jobs")}</p>
                  {detailQuery.query.isFetching ? <PipelineJobsSkeleton /> : null}
                  {!detailQuery.query.isFetching && pipelineJobs.length === 0 ? (
                    <PipelineJobsEmptyState t={t} />
                  ) : null}
                  {pipelineJobs.map((job) => (
                    <PipelineJobCard
                      key={job.pipeline_job.id || job.project_job.id}
                      item={job}
                      active={selectedJobID === job.project_job.id}
                      isCancelling={isCancellingJob}
                      isRetrying={isRetryingJob}
                      canMutate={canMutateJobs}
                      t={t}
                      onInspect={() => setSelectedJobID(job.project_job.id)}
                      onCancel={() => void submitCancelJob(job)}
                      onRetry={() => void submitRetryJob(job)}
                    />
                  ))}
                </div>

                {selectedJob ? (
                  <JobDetailPanel
                    item={selectedJob}
                    trace={trace}
                    artifacts={artifacts}
                    loadingTrace={traceQuery.query.isFetching}
                    loadingArtifacts={artifactsQuery.query.isFetching}
                    t={t}
                    onReload={() => void loadJobDetail()}
                    onDownloadArtifact={(artifact) => void downloadArtifact(artifact)}
                  />
                ) : null}
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const PipelineStat = ({
  label,
  value,
  description,
  status,
}: {
  label: string;
  value: number;
  description: string;
  status: RepositoryPipelineStatus;
}) => (
  <Card className={cn("shadow-none", status === "failed" ? "border-destructive/30 bg-destructive/5" : "bg-muted/20", status === "running" ? "border-primary/30 bg-primary/5" : undefined)}>
    <CardContent className="flex flex-col gap-1 p-3">
      <div className="flex items-center justify-between gap-2">
        <p className="text-xs text-muted-foreground">{label}</p>
        <PipelineStatusBadge status={status} t={(text) => text} />
      </div>
      <p className="text-2xl font-semibold">{value}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </CardContent>
  </Card>
);

const PipelineMeta = ({ label, value }: { label: string; value: string }) => (
  <div className="flex flex-col gap-1 rounded-md border bg-muted/20 p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="truncate text-sm font-medium">{value}</p>
  </div>
);

const PipelineDiagnosticsSummary = ({
  pipeline,
  jobs,
  canMutate,
  t,
}: {
  pipeline: RepositoryPipelineView;
  jobs: RepositoryPipelineJobView[];
  canMutate: boolean;
  t: (text: string) => string;
}) => {
  const summary = getPipelineJobSummary(jobs);
  const problemJobs = jobs.filter((job) => job.status === "failed" || job.status === "blocked").slice(0, 3);
  const hasBlockingIssue = summary.failed > 0 || summary.blocked > 0;

  return (
    <Alert variant={summary.failed > 0 ? "destructive" : "default"} className={cn("bg-muted/20", hasBlockingIssue ? undefined : "border-dashed")}>
      <AlertTitle>{t("Pipeline diagnostics")}</AlertTitle>
      <AlertDescription className="flex flex-col gap-3">
        <div className="grid gap-2 sm:grid-cols-4">
          <PipelineSummaryPill label={t("Jobs")} value={summary.total.toString()} />
          <PipelineSummaryPill label={t("Failed")} value={summary.failed.toString()} urgent={summary.failed > 0} />
          <PipelineSummaryPill label={t("Blocked")}
            value={summary.blocked.toString()}
            urgent={summary.blocked > 0}
          />
          <PipelineSummaryPill label={t("Artifacts expected")} value={summary.artifactJobs.toString()} />
        </div>
        <p>{getPipelineNextAction(pipeline, summary, canMutate, t)}</p>
        {problemJobs.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {problemJobs.map((job) => (
              <Badge key={job.project_job.id} variant="outline" className={job.status === "failed" ? "border-destructive/30 bg-destructive/10 text-destructive" : undefined}>
                {job.pipeline_job.name || job.pipeline_job.stage || job.project_job.id}: {t(job.status)}
              </Badge>
            ))}
          </div>
        ) : null}
      </AlertDescription>
    </Alert>
  );
};

const PipelineSummaryPill = ({ label, value, urgent = false }: { label: string; value: string; urgent?: boolean }) => (
  <div className={cn("rounded-md border bg-background px-3 py-2", urgent ? "border-destructive/30 bg-destructive/10" : undefined)}>
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-sm font-semibold">{value}</p>
  </div>
);

const PipelineEmptyState = ({
  canMutate,
  t,
  onCreate,
}: {
  canMutate: boolean;
  t: (text: string) => string;
  onCreate: () => void;
}) => (
  <Alert className="border-dashed bg-muted/20">
    <AlertTitle>{t("No pipelines yet")}</AlertTitle>
    <AlertDescription className="flex flex-col gap-3">
      <p>{t("Create a manual pipeline to validate CI config, or push commits to let pipelines appear here automatically.")}</p>
      <div className="flex flex-wrap gap-2">
        <Button type="button" size="sm" disabled={!canMutate} onClick={onCreate}>
          <Play data-icon="inline-start" />
          {t("Create first pipeline")}
        </Button>
        {!canMutate ? <Badge variant="outline">{t("CI write permission required")}</Badge> : null}
      </div>
    </AlertDescription>
  </Alert>
);

const PipelinePanelEmptyState = ({ t }: { t: (text: string) => string }) => (
  <Alert className="border-dashed bg-muted/20">
    <AlertTitle>{t("Select a pipeline")}</AlertTitle>
    <AlertDescription>{t("Choose a pipeline from the list to inspect jobs, logs, artifacts, and retry controls.")}</AlertDescription>
  </Alert>
);

const PipelineJobsEmptyState = ({ t }: { t: (text: string) => string }) => (
  <Alert className="border-dashed bg-muted/20">
    <AlertTitle>{t("No jobs in this pipeline")}</AlertTitle>
    <AlertDescription>{t("This pipeline has no materialized jobs yet. Refresh the pipeline state after workers process the config.")}</AlertDescription>
  </Alert>
);

const PipelineListSkeleton = () => (
  <div className="flex flex-col gap-2">
    <Skeleton className="h-20 w-full" />
    <Skeleton className="h-20 w-full" />
    <Skeleton className="h-20 w-full" />
  </div>
);

const PipelineJobsSkeleton = () => (
  <div className="flex flex-col gap-2">
    <Skeleton className="h-24 w-full" />
    <Skeleton className="h-24 w-full" />
  </div>
);

const PipelineJobCard = ({
  item,
  active,
  isCancelling,
  isRetrying,
  canMutate,
  t,
  onInspect,
  onCancel,
  onRetry,
}: {
  item: RepositoryPipelineJobView;
  active: boolean;
  isCancelling: boolean;
  isRetrying: boolean;
  canMutate: boolean;
  t: (text: string) => string;
  onInspect: () => void;
  onCancel: () => void;
  onRetry: () => void;
}) => (
  <div className={cn("flex flex-col gap-3 rounded-md border p-3", active ? "border-primary/60 bg-primary/5" : "bg-background/60", item.status === "failed" ? "border-destructive/30 bg-destructive/5" : undefined)}>
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div className="min-w-0 flex flex-col gap-1">
        <div className="flex flex-wrap items-center gap-2">
          <p className="font-medium">{item.pipeline_job.name || item.pipeline_job.stage || "job"}</p>
          <PipelineJobStatusBadge status={item.status} t={t} />
        </div>
        <p className="text-xs text-muted-foreground">
          {getPipelineJobStatusDescription(item.status, t)}
        </p>
        <p className="text-xs text-muted-foreground">
          {t("Attempts")}: {item.project_job.attempts}/{item.project_job.max_attempts}
          {item.project_job.locked_by ? ` · ${t("Worker")}: ${item.project_job.locked_by}` : ""}
          {item.project_job.updated_at ? ` · ${formatRelativeTime(item.project_job.updated_at)}` : ""}
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant="outline">{item.pipeline_job.stage || t("N/A")}</Badge>
        <Button type="button" size="sm" variant="ghost" onClick={onInspect}>
          <ScrollText data-icon="inline-start" />
          {t("Logs")}
        </Button>
        {!terminalJobStatuses.includes(item.project_job.status) ? (
          <Button type="button" size="sm" variant="outline" disabled={!canMutate || isCancelling} onClick={onCancel}>
            <Ban data-icon="inline-start" />
            {isCancelling ? t("Cancelling...") : t("Cancel")}
          </Button>
        ) : null}
        {item.project_job.status !== "running" ? (
          <Button type="button" size="sm" variant="outline" disabled={!canMutate || isRetrying} onClick={onRetry}>
            <Repeat2 data-icon="inline-start" />
            {isRetrying ? t("Retrying...") : t("Retry")}
          </Button>
        ) : null}
      </div>
    </div>
    {item.needs.length > 0 ? (
      <p className="mt-2 text-xs text-muted-foreground">
        {t("Needs")}: {item.needs.join(", ")}
      </p>
    ) : null}
    {item.tags.length > 0 ? (
      <div className="mt-2 flex flex-wrap gap-2">
        {item.tags.map((tag) => (
          <Badge key={tag} variant="outline">
            {tag}
          </Badge>
        ))}
      </div>
    ) : null}
    {item.script.length > 0 ? (
      <pre className="mt-2 overflow-auto rounded-md bg-muted p-2 text-xs">{item.script.join("\n")}</pre>
    ) : null}
    {item.artifacts.length > 0 ? (
      <p className="mt-2 text-xs text-muted-foreground">
        {t("Artifacts")}: {item.artifacts.join(", ")}
      </p>
    ) : null}
    {item.status === "failed" || item.status === "blocked" ? (
      <Alert variant={item.status === "failed" ? "destructive" : "default"} className="mt-2 bg-muted/20 px-2 py-1 text-xs">
        <AlertDescription className="text-xs leading-5">
          {getPipelineJobNextAction(item, t)}
        </AlertDescription>
      </Alert>
    ) : null}
    {item.project_job.last_error ? (
      <Alert variant="destructive" className="mt-2 px-2 py-1 text-xs">
        <AlertDescription className="text-xs leading-5">{item.project_job.last_error}</AlertDescription>
      </Alert>
    ) : null}
  </div>
);

const JobDetailPanel = ({
  item,
  trace,
  artifacts,
  loadingTrace,
  loadingArtifacts,
  t,
  onReload,
  onDownloadArtifact,
}: {
  item: RepositoryPipelineJobView;
  trace: RepositoryJobTraceView | null;
  artifacts: RepositoryJobArtifactView[];
  loadingTrace: boolean;
  loadingArtifacts: boolean;
  t: (text: string) => string;
  onReload: () => void;
  onDownloadArtifact: (artifact: RepositoryJobArtifactView) => void;
}) => (
  <div className="rounded-md border bg-muted/10 p-3">
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div>
        <div className="flex flex-wrap items-center gap-2">
          <ScrollText className="size-4 text-muted-foreground" />
          <p className="font-medium">
            {t("Job detail")}: {item.pipeline_job.name || item.pipeline_job.stage || item.project_job.id}
          </p>
          <PipelineJobStatusBadge status={item.status} t={t} />
        </div>
        <p className="mt-1 text-xs text-muted-foreground">
          {t("Job ID")}: {item.project_job.id} · {t("Duration")}: {trace?.duration_millis ?? 0}ms · {t("Exit code")}: {trace?.exit_code ?? 0}
        </p>
      </div>
      <Button type="button" size="sm" variant="ghost" onClick={onReload}>
        <RefreshCw data-icon="inline-start" />
        {t("Reload job")}
      </Button>
    </div>

    <div className="mt-3 grid gap-3 xl:grid-cols-[minmax(0,1fr)_280px]">
      <div className="min-w-0">
        <p className="mb-2 text-sm font-medium">{t("Trace")}</p>
        {loadingTrace ? (
          <Skeleton className="h-48 w-full" />
        ) : trace?.trace || item.project_job.last_error ? (
          <pre className="max-h-96 overflow-auto rounded-md bg-background p-3 text-xs">
            {trace?.trace || item.project_job.last_error}
          </pre>
        ) : (
          <Alert className="border-dashed bg-background">
            <AlertTitle>{t("No trace captured yet")}</AlertTitle>
            <AlertDescription>{getPipelineTraceEmptyMessage(item.status, t)}</AlertDescription>
          </Alert>
        )}
      </div>
      <div>
        <p className="mb-2 flex items-center gap-2 text-sm font-medium">
          <FileArchive className="size-4" />
          {t("Artifacts")}
        </p>
        {loadingArtifacts ? <Skeleton className="h-24 w-full" /> : null}
        {!loadingArtifacts && artifacts.length === 0 ? (
          <Alert className="border-dashed bg-muted/20">
            <AlertTitle>{t("No artifacts available")}</AlertTitle>
            <AlertDescription>{getPipelineArtifactEmptyMessage(item, t)}</AlertDescription>
          </Alert>
        ) : null}
        <div className="flex flex-col gap-2">
          {artifacts.map((artifact) => (
            <div key={artifact.id} className="rounded-md border bg-background p-2">
              <p className="truncate text-sm font-medium">{artifact.file_name || artifact.name || "artifact"}</p>
              <p className="text-xs text-muted-foreground">{formatBytes(artifact.byte_size)}</p>
              <Button type="button" size="sm" variant="ghost" className="mt-2 w-full justify-start" onClick={() => onDownloadArtifact(artifact)}>
                <Download data-icon="inline-start" />
                {t("Download")}
              </Button>
            </div>
          ))}
        </div>
      </div>
    </div>
  </div>
);

const PipelineStatusBadge = ({ status, t }: { status: RepositoryPipelineStatus; t: (text: string) => string }) => {
  const icon = {
    pending: <Clock3 className="size-3" />,
    running: <Loader2 className="size-3 animate-spin" />,
    succeeded: <CheckCircle2 className="size-3" />,
    failed: <XCircle className="size-3" />,
    cancelled: <Ban className="size-3" />,
  }[status];
  const variant = status === "succeeded" ? "default" : status === "pending" || status === "running" ? "secondary" : "outline";
  return (
    <Badge variant={variant} className="gap-1">
      {icon}
      {t(status)}
    </Badge>
  );
};

const getPipelineJobStatusDescription = (status: RepositoryPipelineJobStatus, t: (text: string) => string): string => {
  const descriptions: Record<RepositoryPipelineJobStatus, string> = {
    pending: t("Waiting for dependencies or an available runner."),
    running: t("Currently executing on a runner."),
    succeeded: t("Completed successfully."),
    failed: t("Failed and needs log review."),
    cancelled: t("Cancelled before completion."),
    blocked: t("Blocked by upstream job dependencies."),
  };
  return descriptions[status];
};

interface PipelineJobSummary {
  total: number;
  pending: number;
  running: number;
  succeeded: number;
  failed: number;
  cancelled: number;
  blocked: number;
  artifactJobs: number;
}

const getPipelineJobSummary = (jobs: RepositoryPipelineJobView[]): PipelineJobSummary =>
  jobs.reduce<PipelineJobSummary>(
    (summary, job) => ({
      total: summary.total + 1,
      pending: summary.pending + (job.status === "pending" ? 1 : 0),
      running: summary.running + (job.status === "running" ? 1 : 0),
      succeeded: summary.succeeded + (job.status === "succeeded" ? 1 : 0),
      failed: summary.failed + (job.status === "failed" ? 1 : 0),
      cancelled: summary.cancelled + (job.status === "cancelled" ? 1 : 0),
      blocked: summary.blocked + (job.status === "blocked" ? 1 : 0),
      artifactJobs: summary.artifactJobs + (job.artifacts.length > 0 ? 1 : 0),
    }),
    { total: 0, pending: 0, running: 0, succeeded: 0, failed: 0, cancelled: 0, blocked: 0, artifactJobs: 0 },
  );

const getPipelineNextAction = (
  pipeline: RepositoryPipelineView,
  summary: PipelineJobSummary,
  canMutate: boolean,
  t: (text: string) => string,
): string => {
  if (summary.failed > 0) {
    return canMutate
      ? t("Next action: open the failed job log, compare the exit code with the script, download any artifacts, then retry only the failed stage after fixing the cause.")
      : t("Next action: open the failed job log and artifacts, then ask a CI maintainer to retry after the cause is fixed.");
  }
  if (summary.blocked > 0) {
    return t("Next action: inspect the blocked jobs' Needs list and upstream stages. A blocked job usually waits for a dependency to finish or for a failed dependency to be retried.");
  }
  if (summary.pending > 0 && summary.running === 0) {
    return t("Next action: check runner health and tag matching. Queued jobs stay pending when no active runner can claim their tags.");
  }
  if (pipeline.status === "running" || summary.running > 0) {
    return t("Next action: watch the active job trace and reload artifacts after the job finishes.");
  }
  if (pipeline.status === "succeeded") {
    return t("Pipeline completed. Review artifacts from the producing jobs before promoting the commit.");
  }
  if (pipeline.status === "cancelled") {
    return canMutate ? t("Pipeline was cancelled. Retry it if the ref and CI config are still valid.") : t("Pipeline was cancelled. Ask a CI maintainer to retry it if needed.");
  }
  return t("No blocking diagnosis yet. Refresh pipeline state after runners process the config.");
};

const getPipelineJobNextAction = (job: RepositoryPipelineJobView, t: (text: string) => string): string => {
  if (job.status === "failed") {
    return job.artifacts.length > 0
      ? t("Next action: inspect the trace around the first failing command, then download artifacts for reports, coverage, screenshots, or build output.")
      : t("Next action: inspect the trace around the first failing command. No artifact pattern is declared for this job.");
  }
  return job.needs.length > 0
    ? t("Next action: check upstream Needs jobs first. This job will remain blocked until required dependencies complete successfully.")
    : t("Next action: refresh pipeline state. If it stays blocked, check runner capacity and tag matching.");
};

const getPipelineTraceEmptyMessage = (status: RepositoryPipelineJobStatus, t: (text: string) => string): string => {
  if (status === "pending" || status === "blocked") {
    return t("Trace is empty because the job has not run yet. Check dependencies, runner availability, and matching tags.");
  }
  if (status === "running") {
    return t("Trace may still be streaming. Reload the job after the runner flushes output.");
  }
  return t("No trace was stored for this job. Use the last error, exit code, and artifacts to continue diagnosis.");
};

const getPipelineArtifactEmptyMessage = (job: RepositoryPipelineJobView, t: (text: string) => string): string => {
  if (job.artifacts.length === 0) {
    return t("This job does not declare artifact paths in the CI config, so no downloadable files are expected.");
  }
  if (job.status === "failed") {
    return t("Artifact paths were declared, but nothing is available. Check whether the script failed before producing files or whether upload failed.");
  }
  return t("Artifact paths were declared, but no files have been uploaded yet. Reload after the job completes.");
};

const getPipelineActivitySummary = (pipelines: RepositoryPipelineView[], t: (text: string) => string): string => {
  if (pipelines.length === 0) {
    return t("No pipeline history yet. Create a manual pipeline or push a commit to start collecting CI diagnostics.");
  }
  const newest = pipelines[0];
  if (newest.status === "failed") {
    return t("Latest pipeline failed. Select it to inspect failed jobs, trace output, artifacts, and retry options.");
  }
  if (newest.status === "running") {
    return t("Latest pipeline is running. Watch active job traces and refresh artifacts after completion.");
  }
  if (newest.status === "pending") {
    return t("Latest pipeline is queued. Check runner health and tag matching if it does not start soon.");
  }
  if (newest.status === "succeeded") {
    return t("Latest pipeline succeeded. Use job artifacts and logs as the release evidence trail.");
  }
  return t("Latest pipeline was cancelled. Retry it only if the ref and CI config still represent the intended run.");
};

const PipelineJobStatusBadge = ({ status, t }: { status: RepositoryPipelineJobStatus; t: (text: string) => string }) => {
  if (status === "blocked") {
    return (
      <Badge variant="outline" className="gap-1">
        <Clock3 className="size-3" />
        {t("blocked")}
      </Badge>
    );
  }
  return <PipelineStatusBadge status={status} t={t} />;
};

const resolvePipelineList = (payload: unknown): RawRecord[] => {
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

const normalizePipelineDetail = (payload: unknown): RepositoryPipelineDetailView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    pipeline: normalizePipeline(raw.pipeline ?? raw.Pipeline),
    jobs: resolveRecordArray(raw.jobs ?? raw.Jobs).map(normalizePipelineJob),
  };
};

const normalizePipeline = (rawValue: unknown): RepositoryPipelineView => {
  const raw = isRecord(rawValue) ? rawValue : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    iid: normalizeNumber(raw.iid ?? raw.IID),
    name: normalizeString(raw.name ?? raw.Name),
    source: normalizeString(raw.source ?? raw.Source),
    ref_name: normalizeString(raw.ref_name ?? raw.RefName),
    commit_sha: normalizeString(raw.commit_sha ?? raw.CommitSHA),
    status: normalizePipelineStatus(raw.status ?? raw.Status),
    config_source: normalizeString(raw.config_source ?? raw.ConfigSource),
    config_content: normalizeOptionalString(raw.config_content ?? raw.ConfigContent),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
    started_at: normalizeOptionalString(raw.started_at ?? raw.StartedAt),
    finished_at: normalizeOptionalString(raw.finished_at ?? raw.FinishedAt),
  };
};

const normalizePipelineJob = (rawValue: unknown): RepositoryPipelineJobView => {
  const raw = isRecord(rawValue) ? rawValue : {};
  return {
    pipeline_job: normalizePipelineJobLink(raw.pipeline_job ?? raw.PipelineJob),
    project_job: normalizeJob(raw.project_job ?? raw.ProjectJob),
    status: normalizePipelineJobStatus(raw.status ?? raw.Status),
    needs: normalizeStringArray(raw.needs ?? raw.Needs),
    script: normalizeStringArray(raw.script ?? raw.Script),
    artifacts: normalizeStringArray(raw.artifacts ?? raw.Artifacts),
    tags: normalizeStringArray(raw.tags ?? raw.Tags),
  };
};

const normalizeJobTrace = (payload: unknown): RepositoryJobTraceView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    job: normalizeJob(raw.job ?? raw.Job),
    trace: normalizeString(raw.trace ?? raw.Trace),
    exit_code: normalizeNumber(raw.exit_code ?? raw.ExitCode),
    output_truncated: normalizeBoolean(raw.output_truncated ?? raw.OutputTruncated),
    duration_millis: normalizeNumber(raw.duration_millis ?? raw.DurationMillis),
  };
};

const resolveArtifactList = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (Array.isArray(raw)) {
    return raw.filter(isRecord);
  }
  if (!isRecord(raw)) {
    return [];
  }
  return resolveRecordArray(raw.artifacts ?? raw.Artifacts);
};

const normalizeArtifact = (rawValue: unknown): RepositoryJobArtifactView => {
  const raw = isRecord(rawValue) ? rawValue : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    project_job_id: normalizeString(raw.project_job_id ?? raw.ProjectJobID),
    name: normalizeString(raw.name ?? raw.Name),
    file_name: normalizeString(raw.file_name ?? raw.FileName),
    file_path: normalizeOptionalString(raw.file_path ?? raw.FilePath),
    content_type: normalizeOptionalString(raw.content_type ?? raw.ContentType),
    byte_size: normalizeNumber(raw.byte_size ?? raw.ByteSize),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const normalizeArtifactContent = (payload: unknown): RepositoryJobArtifactContentView => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : {};
  const record = isRecord(raw) ? raw : {};
  return {
    artifact: normalizeArtifact(record.artifact ?? record.Artifact),
    content_base64: normalizeString(record.content_base64 ?? record.ContentBase64),
  };
};

const normalizePipelineJobLink = (rawValue: unknown) => {
  const raw = isRecord(rawValue) ? rawValue : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    pipeline_id: normalizeString(raw.pipeline_id ?? raw.PipelineID),
    project_job_id: normalizeString(raw.project_job_id ?? raw.ProjectJobID),
    name: normalizeString(raw.name ?? raw.Name),
    stage: normalizeString(raw.stage ?? raw.Stage),
    needs: normalizeOptionalString(raw.needs ?? raw.Needs),
    image: normalizeOptionalString(raw.image ?? raw.Image),
    script: normalizeOptionalString(raw.script ?? raw.Script),
    artifacts: normalizeOptionalString(raw.artifacts ?? raw.Artifacts),
    sort_order: normalizeNumber(raw.sort_order ?? raw.SortOrder),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const normalizeJob = (rawValue: unknown): RepositoryJobView => {
  const raw = isRecord(rawValue) ? rawValue : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    kind: normalizeString(raw.kind ?? raw.Kind) || "noop",
    status: normalizeJobStatus(raw.status ?? raw.Status),
    payload: normalizeOptionalString(raw.payload ?? raw.Payload),
    result: normalizeOptionalString(raw.result ?? raw.Result),
    attempts: normalizeNumber(raw.attempts ?? raw.Attempts),
    max_attempts: normalizeNumber(raw.max_attempts ?? raw.MaxAttempts) || 1,
    run_after: normalizeOptionalString(raw.run_after ?? raw.RunAfter),
    locked_by: normalizeOptionalString(raw.locked_by ?? raw.LockedBy),
    locked_until: normalizeOptionalString(raw.locked_until ?? raw.LockedUntil),
    last_error: normalizeOptionalString(raw.last_error ?? raw.LastError),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
    started_at: normalizeOptionalString(raw.started_at ?? raw.StartedAt),
    finished_at: normalizeOptionalString(raw.finished_at ?? raw.FinishedAt),
  };
};

const normalizePipelineStatus = (value: unknown): RepositoryPipelineStatus => {
  const normalized = normalizeString(value);
  if (normalized === "canceled") {
    return "cancelled";
  }
  if (normalized === "pending" || normalized === "running" || normalized === "succeeded" || normalized === "failed" || normalized === "cancelled") {
    return normalized;
  }
  return "pending";
};

const normalizePipelineJobStatus = (value: unknown): RepositoryPipelineJobStatus => {
  const normalized = normalizeString(value);
  if (normalized === "blocked") {
    return "blocked";
  }
  return normalizeJobStatus(value);
};

const normalizeJobStatus = (value: unknown): RepositoryJobStatus => {
  const normalized = normalizeString(value);
  if (normalized === "canceled") {
    return "cancelled";
  }
  if (normalized === "pending" || normalized === "running" || normalized === "succeeded" || normalized === "failed" || normalized === "cancelled") {
    return normalized;
  }
  return "pending";
};

const shortSHA = (value: string): string => {
  const normalized = value.trim();
  return normalized ? normalized.slice(0, 8) : "unknown";
};

const decodeBase64 = (value: string): ArrayBuffer => {
  const binary = window.atob(value);
  const buffer = new ArrayBuffer(binary.length);
  const bytes = new Uint8Array(buffer);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return buffer;
};

const formatBytes = (value: number): string => {
  if (!Number.isFinite(value) || value <= 0) {
    return "0 B";
  }
  const units = ["B", "KB", "MB", "GB", "TB"];
  const exponent = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  const amount = value / 1024 ** exponent;
  return `${amount >= 10 || exponent === 0 ? amount.toFixed(0) : amount.toFixed(1)} ${units[exponent]}`;
};
