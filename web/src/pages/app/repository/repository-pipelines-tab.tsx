import { useEffect, useMemo, useState } from "react";
import { Ban, CheckCircle2, Clock3, Download, FileArchive, GitBranch, Loader2, Play, RefreshCw, Repeat2, ScrollText, XCircle } from "lucide-react";
import { useCustom, useCustomMutation, useDataProvider } from "@refinedev/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
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
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-4">
          <PipelineStat label={t("Pending")} value={stats.pending} tone="amber" />
          <PipelineStat label={t("Running")} value={stats.running} tone="blue" />
          <PipelineStat label={t("Succeeded")} value={stats.succeeded} tone="emerald" />
          <PipelineStat label={t("Failed")} value={stats.failed} tone="rose" />
        </div>

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
            <Play className="size-4" />
            {isComposerOpen ? t("Hide pipeline form") : t("New pipeline")}
          </Button>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="secondary">
              {t("Role")}: {t(permissions.roleLabel)}
            </Badge>
            <Button type="button" size="sm" variant="ghost" onClick={() => void loadPipelines()}>
              <RefreshCw className="size-4" />
              {t("Reload")}
            </Button>
          </div>
        </div>

        {isComposerOpen ? (
          <form className="space-y-3 rounded-md border bg-muted/10 p-3" onSubmit={submitCreatePipeline}>
            <div className="grid gap-3 md:grid-cols-[160px_1fr_1fr]">
              <div className="space-y-1">
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
              <div className="space-y-1">
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
              <div className="space-y-1">
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
            <div className="space-y-1">
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
            <div className="space-y-1">
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
          <div className="space-y-3">
            <div className="flex items-center justify-between gap-2">
              <p className="text-sm font-medium">{t("Pipelines")}</p>
            </div>
            <div className="space-y-2 rounded-md border p-2">
              {isLoadingPipelines ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading pipelines...")}</p> : null}
              {!isLoadingPipelines && pipelines.length === 0 ? (
                <p className="px-2 py-2 text-sm text-muted-foreground">{t("No pipelines found.")}</p>
              ) : null}
              {pipelines.map((pipeline) => (
                <button
                  key={pipeline.id}
                  type="button"
                  className={`w-full rounded-md border p-3 text-left transition hover:bg-muted/40 ${
                    visiblePipeline?.id === pipeline.id ? "border-primary/60 bg-primary/5" : ""
                  }`}
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
              <p className="text-sm text-muted-foreground">{t("Select a pipeline to inspect jobs.")}</p>
            ) : (
              <div className="space-y-4">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0 space-y-1">
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
                      <RefreshCw className="size-4" />
                      {t("Reload pipeline")}
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={!canMutateCI || isRefreshingPipeline}
                      onClick={() => void submitRefreshPipeline()}
                    >
                      <RefreshCw className="size-4" />
                      {isRefreshingPipeline ? t("Refreshing...") : t("Refresh state")}
                    </Button>
                    {!terminalPipelineStatuses.includes(visiblePipeline.status) ? (
                      <Button type="button" size="sm" variant="outline" disabled={!canMutateCI || isCancelling} onClick={() => void submitCancelPipeline()}>
                        <Ban className="size-4" />
                        {isCancelling ? t("Cancelling...") : t("Cancel pipeline")}
                      </Button>
                    ) : null}
                    {visiblePipeline.status !== "running" ? (
                      <Button type="button" size="sm" variant="outline" disabled={!canMutateCI || isRetrying} onClick={() => void submitRetryPipeline()}>
                        <Repeat2 className="size-4" />
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

                <div className="space-y-2">
                  <p className="text-sm font-medium">{t("Pipeline jobs")}</p>
                  {detailQuery.query.isFetching ? (
                    <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("Loading jobs...")}</p>
                  ) : null}
                  {!detailQuery.query.isFetching && pipelineJobs.length === 0 ? (
                    <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("No pipeline jobs found.")}</p>
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

const PipelineStat = ({ label, value, tone }: { label: string; value: number; tone: "amber" | "blue" | "emerald" | "rose" }) => {
  const toneClass = {
    amber: "border-amber-500/30 bg-amber-500/5",
    blue: "border-blue-500/30 bg-blue-500/5",
    emerald: "border-emerald-500/30 bg-emerald-500/5",
    rose: "border-rose-500/30 bg-rose-500/5",
  }[tone];

  return (
    <div className={`rounded-md border p-3 ${toneClass}`}>
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="text-lg font-semibold">{value}</p>
    </div>
  );
};

const PipelineMeta = ({ label, value }: { label: string; value: string }) => (
  <div className="rounded-md border bg-muted/20 p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="mt-1 truncate text-sm font-medium">{value}</p>
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
  <div className={`rounded-md border p-3 ${active ? "border-primary/60 bg-primary/5" : ""}`}>
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div className="min-w-0 space-y-1">
        <div className="flex flex-wrap items-center gap-2">
          <p className="font-medium">{item.pipeline_job.name || item.pipeline_job.stage || "job"}</p>
          <PipelineJobStatusBadge status={item.status} t={t} />
        </div>
        <p className="text-xs text-muted-foreground">
          {t("Attempts")}: {item.project_job.attempts}/{item.project_job.max_attempts}
          {item.project_job.locked_by ? ` · ${t("Worker")}: ${item.project_job.locked_by}` : ""}
          {item.project_job.updated_at ? ` · ${formatRelativeTime(item.project_job.updated_at)}` : ""}
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant="outline">{item.pipeline_job.stage || t("N/A")}</Badge>
        <Button type="button" size="sm" variant="ghost" onClick={onInspect}>
          <ScrollText className="size-4" />
          {t("Logs")}
        </Button>
        {!terminalJobStatuses.includes(item.project_job.status) ? (
          <Button type="button" size="sm" variant="outline" disabled={!canMutate || isCancelling} onClick={onCancel}>
            <Ban className="size-4" />
            {isCancelling ? t("Cancelling...") : t("Cancel")}
          </Button>
        ) : null}
        {item.project_job.status !== "running" ? (
          <Button type="button" size="sm" variant="outline" disabled={!canMutate || isRetrying} onClick={onRetry}>
            <Repeat2 className="size-4" />
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
        <RefreshCw className="size-4" />
        {t("Reload job")}
      </Button>
    </div>

    <div className="mt-3 grid gap-3 xl:grid-cols-[minmax(0,1fr)_280px]">
      <div className="min-w-0">
        <p className="mb-2 text-sm font-medium">{t("Trace")}</p>
        {loadingTrace ? (
          <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("Loading trace...")}</p>
        ) : (
          <pre className="max-h-96 overflow-auto rounded-md bg-background p-3 text-xs">
            {trace?.trace || item.project_job.last_error || t("No trace output.")}
          </pre>
        )}
      </div>
      <div>
        <p className="mb-2 flex items-center gap-2 text-sm font-medium">
          <FileArchive className="size-4" />
          {t("Artifacts")}
        </p>
        {loadingArtifacts ? (
          <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("Loading artifacts...")}</p>
        ) : null}
        {!loadingArtifacts && artifacts.length === 0 ? (
          <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("No artifacts uploaded.")}</p>
        ) : null}
        <div className="space-y-2">
          {artifacts.map((artifact) => (
            <div key={artifact.id} className="rounded-md border bg-background p-2">
              <p className="truncate text-sm font-medium">{artifact.file_name || artifact.name || "artifact"}</p>
              <p className="text-xs text-muted-foreground">{formatBytes(artifact.byte_size)}</p>
              <Button type="button" size="sm" variant="ghost" className="mt-2 w-full justify-start" onClick={() => onDownloadArtifact(artifact)}>
                <Download className="size-4" />
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
    pending: <Clock3 className="mr-1 size-3" />,
    running: <Loader2 className="mr-1 size-3 animate-spin" />,
    succeeded: <CheckCircle2 className="mr-1 size-3" />,
    failed: <XCircle className="mr-1 size-3" />,
    cancelled: <Ban className="mr-1 size-3" />,
  }[status];
  const variant = status === "succeeded" ? "default" : status === "pending" || status === "running" ? "secondary" : "outline";
  return (
    <Badge variant={variant}>
      {icon}
      {t(status)}
    </Badge>
  );
};

const PipelineJobStatusBadge = ({ status, t }: { status: RepositoryPipelineJobStatus; t: (text: string) => string }) => {
  if (status === "blocked") {
    return (
      <Badge variant="outline">
        <Clock3 className="mr-1 size-3" />
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
