import { useEffect, useMemo, useState } from "react";
import { Ban, CheckCircle2, Clock3, FileArchive, Loader2, Play, RefreshCw, ScrollText, XCircle } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";
import type { RepositoryJobStatus, RepositoryJobView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeNumber, normalizeOptionalString, normalizeString, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryJobsTabProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

type RawProjectJob = RawRecord;

const terminalStatuses: RepositoryJobStatus[] = ["succeeded", "failed", "cancelled"];

export const RepositoryJobsTab = ({ repoId, permissions, t, onError }: RepositoryJobsTabProps): JSX.Element => {
  const jobsQuery = useCustom<RawProjectJob[]>({
    url: `/projects/${repoId}/jobs`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: enqueueJob, mutation: { isPending: isEnqueueing } } = useCustomMutation<RawProjectJob>();
  const { mutateAsync: cancelJob, mutation: { isPending: isCancelling } } = useCustomMutation<RawProjectJob>();
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [kind, setKind] = useState("noop");
  const [payload, setPayload] = useState("");
  const [maxAttempts, setMaxAttempts] = useState("3");
  const canMutateJobs = permissions.jobWrite;

  const jobs = useMemo(
    () => resolveJobList(jobsQuery.result.data).map(normalizeJob),
    [jobsQuery.result.data],
  );
  const stats = useMemo(
    () => ({
      pending: jobs.filter((item) => item.status === "pending").length,
      running: jobs.filter((item) => item.status === "running").length,
      failed: jobs.filter((item) => item.status === "failed").length,
      completed: jobs.filter((item) => item.status === "succeeded").length,
    }),
    [jobs],
  );
  const isLoadingJobs = jobsQuery.query.isFetching && !jobsQuery.query.data;

  const loadJobs = async () => {
    const result = await jobsQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const submitEnqueueJob = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedKind = kind.trim() || "noop";
    const normalizedMaxAttempts = Number.parseInt(maxAttempts, 10);
    if (!Number.isFinite(normalizedMaxAttempts) || normalizedMaxAttempts <= 0) {
      onError(t("Max attempts must be a positive number"));
      return;
    }

    onError(null);
    try {
      await enqueueJob({
        url: `/projects/${repoId}/jobs`,
        method: "post",
        values: {
          kind: normalizedKind,
          payload: payload.trim(),
          max_attempts: normalizedMaxAttempts,
        },
      });
      setComposerOpen(false);
      setKind("noop");
      setPayload("");
      setMaxAttempts("3");
      await loadJobs();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitCancelJob = async (job: RepositoryJobView) => {
    onError(null);
    try {
      await cancelJob({
        url: `/projects/${repoId}/jobs/${job.id}/cancel`,
        method: "post",
        values: {},
      });
      await loadJobs();
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
    if (!jobsQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(jobsQuery.query.error));
  }, [jobsQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Jobs")}</CardTitle>
        <CardDescription>{t("Inspect and enqueue project background jobs.")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-4">
          <JobStat label={t("Pending")} value={stats.pending} description={stats.pending > 0 ? t("Waiting for a runner") : t("Queue is clear")} status="pending" />
          <JobStat label={t("Running")} value={stats.running} description={stats.running > 0 ? t("Currently executing") : t("No active jobs")} status="running" />
          <JobStat label={t("Succeeded")} value={stats.completed} description={stats.completed > 0 ? t("Completed successfully") : t("No successful jobs yet")} status="succeeded" />
          <JobStat label={t("Failed")} value={stats.failed} description={stats.failed > 0 ? t("Needs attention") : t("No failures")} status="failed" />
        </div>

        <JobQueueDiagnostics stats={stats} t={t} />

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isComposerOpen ? "secondary" : "outline"}
            disabled={!canMutateJobs}
            onClick={() => setComposerOpen((current) => !current)}
          >
            <Play data-icon="inline-start" />
            {isComposerOpen ? t("Hide new job form") : t("New job")}
          </Button>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={canMutateJobs ? "secondary" : "outline"}>
              {t("Role")}: {t(permissions.roleLabel)}
            </Badge>
            <Button type="button" size="sm" variant="ghost" onClick={() => void loadJobs()}>
              <RefreshCw data-icon="inline-start" />
              {t("Reload")}
            </Button>
          </div>
        </div>

        {!canMutateJobs ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect jobs, but cannot enqueue or cancel them.")}</AlertDescription>
          </Alert>
        ) : null}

        {isComposerOpen ? (
          <form className="flex flex-col gap-3 rounded-md border bg-muted/10 p-3" onSubmit={submitEnqueueJob}>
            <div className="grid gap-3 md:grid-cols-[1fr_160px]">
              <div className="flex flex-col gap-1">
                <Label className="text-xs text-muted-foreground" htmlFor="job-kind">
                  {t("Job kind")}
                </Label>
                <Input
                  id="job-kind"
                  value={kind}
                  onChange={(event) => setKind(event.target.value)}
                  placeholder="noop"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label className="text-xs text-muted-foreground" htmlFor="job-max-attempts">
                  {t("Max attempts")}
                </Label>
                <Input
                  id="job-max-attempts"
                  type="number"
                  min={1}
                  value={maxAttempts}
                  onChange={(event) => setMaxAttempts(event.target.value)}
                />
              </div>
            </div>
            <div className="flex flex-col gap-1">
              <Label className="text-xs text-muted-foreground" htmlFor="job-payload">
                {t("Payload JSON")}
              </Label>
              <Textarea
                id="job-payload"
                className="min-h-24"
                value={payload}
                onChange={(event) => setPayload(event.target.value)}
                placeholder='{"reason":"manual"}'
              />
            </div>
            <div className="flex justify-end">
              <Button type="submit" disabled={!canMutateJobs || isEnqueueing}>
                {isEnqueueing ? t("Enqueueing job...") : t("Enqueue job")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="flex flex-col gap-2 rounded-md border p-2">
          {isLoadingJobs ? <JobListSkeleton /> : null}
          {!isLoadingJobs && jobs.length === 0 ? (
            <JobEmptyState
              canMutate={canMutateJobs}
              t={t}
              onCreate={() => setComposerOpen(true)}
            />
          ) : null}
          {jobs.map((job) => (
            <div
              key={job.id}
              className={cn(
                "flex flex-col gap-3 rounded-md border p-3",
                job.status === "failed" ? "border-destructive/30 bg-destructive/5" : "bg-background/60",
                job.status === "running" ? "border-primary/30 bg-primary/5" : undefined,
              )}
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="flex min-w-0 flex-col gap-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="truncate font-medium">#{job.id}</p>
                    <Badge variant="outline">{job.kind}</Badge>
                    <JobStatusBadge status={job.status} t={t} />
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {getJobStatusDescription(job.status, t)}
                  </p>
                </div>
                {!terminalStatuses.includes(job.status) ? (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={!canMutateJobs || isCancelling}
                    onClick={() => void submitCancelJob(job)}
                  >
                    <Ban data-icon="inline-start" />
                    {t("Cancel job")}
                  </Button>
                ) : null}
              </div>
              <div className="grid gap-2 md:grid-cols-3">
                <JobMeta label={t("Attempts")} value={`${job.attempts}/${job.max_attempts}`} />
                <JobMeta label={t("Worker")} value={job.locked_by || t("Unassigned")} />
                <JobMeta label={t("Updated")} value={job.updated_at ? formatRelativeTime(job.updated_at) : t("Not updated")} />
              </div>
              {job.last_error ? (
                <Alert variant="destructive" className="px-2 py-1 text-xs">
                  <AlertDescription className="text-xs leading-5">{job.last_error}</AlertDescription>
                </Alert>
              ) : null}
              <JobOutputDiagnostics job={job} t={t} />
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

const JobStat = ({
  label,
  value,
  description,
  status,
}: {
  label: string;
  value: number;
  description: string;
  status: RepositoryJobStatus;
}) => (
  <Card className={cn("shadow-none", status === "failed" ? "border-destructive/30 bg-destructive/5" : "bg-muted/20", status === "running" ? "border-primary/30 bg-primary/5" : undefined)}>
    <CardContent className="flex flex-col gap-1 p-3">
      <div className="flex items-center justify-between gap-2">
        <p className="text-xs text-muted-foreground">{label}</p>
        <JobStatusBadge status={status} t={(text) => text} />
      </div>
      <p className="text-2xl font-semibold">{value}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </CardContent>
  </Card>
);

const JobMeta = ({ label, value }: { label: string; value: string }) => (
  <div className="rounded-md border bg-muted/20 px-3 py-2">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="truncate text-sm font-medium">{value}</p>
  </div>
);

interface JobQueueStats {
  pending: number;
  running: number;
  failed: number;
  completed: number;
}

const JobQueueDiagnostics = ({ stats, t }: { stats: JobQueueStats; t: (text: string) => string }) => (
  <Alert className="bg-muted/20">
    <AlertTitle>{t("Job diagnostics guide")}</AlertTitle>
    <AlertDescription className="flex flex-col gap-2">
      <p>{getJobQueueDiagnosticMessage(stats, t)}</p>
      <div className="flex flex-wrap gap-2">
        <Badge variant={stats.failed > 0 ? "secondary" : "outline"}>{t("Trace")}: {t("pipeline job detail")}</Badge>
        <Badge variant="outline">{t("Artifacts")}: {t("download after upload")}</Badge>
        <Badge variant="outline">{t("Empty logs")}: {t("not claimed, not flushed, or no stored output")}</Badge>
      </div>
    </AlertDescription>
  </Alert>
);

const JobOutputDiagnostics = ({ job, t }: { job: RepositoryJobView; t: (text: string) => string }) => (
  <div className="flex flex-col gap-2 rounded-md border bg-muted/10 p-2">
    <div className="flex flex-wrap items-center justify-between gap-2">
      <p className="flex items-center gap-2 text-xs font-medium">
        <ScrollText className="size-4 text-muted-foreground" />
        {t("Trace and artifacts")}
      </p>
      <div className="flex flex-wrap gap-2">
        <Badge variant="outline">{t("Job ID")}: {job.id}</Badge>
        <Badge variant="outline">{t("Attempts")}: {job.attempts}/{job.max_attempts}</Badge>
      </div>
    </div>
    <p className="text-xs text-muted-foreground">{getJobTraceArtifactGuidance(job, t)}</p>
    {job.result ? (
      <pre className="overflow-auto rounded-md bg-background p-2 text-xs">{job.result}</pre>
    ) : (
      <Alert className="border-dashed bg-background">
        <AlertTitle>{t("No stored job output")}</AlertTitle>
        <AlertDescription>{getJobOutputEmptyMessage(job.status, t)}</AlertDescription>
      </Alert>
    )}
    <p className="flex items-center gap-2 text-xs text-muted-foreground">
      <FileArchive className="size-4" />
      {t("Artifacts are produced by CI script jobs. If none appear, confirm the pipeline job declares artifact paths and the script reaches upload.")}
    </p>
  </div>
);

const JobEmptyState = ({
  canMutate,
  t,
  onCreate,
}: {
  canMutate: boolean;
  t: (text: string) => string;
  onCreate: () => void;
}) => (
  <Alert className="border-dashed bg-muted/20">
    <AlertTitle>{t("No jobs in the queue")}</AlertTitle>
    <AlertDescription className="flex flex-col gap-3">
      <p>{t("Enqueue a manual job to validate runner connectivity, or wait for a pipeline to create jobs automatically.")}</p>
      <div className="flex flex-wrap gap-2">
        <Button type="button" size="sm" disabled={!canMutate} onClick={onCreate}>
          <Play data-icon="inline-start" />
          {t("Enqueue first job")}
        </Button>
        {!canMutate ? <Badge variant="outline">{t("Read-only role")}</Badge> : null}
      </div>
    </AlertDescription>
  </Alert>
);

const JobListSkeleton = () => (
  <div className="flex flex-col gap-2">
    <Skeleton className="h-24 w-full" />
    <Skeleton className="h-24 w-full" />
    <Skeleton className="h-24 w-full" />
  </div>
);

const getJobStatusDescription = (status: RepositoryJobStatus, t: (text: string) => string): string => {
  const descriptions: Record<RepositoryJobStatus, string> = {
    pending: t("Queued and waiting for an available runner."),
    running: t("Claimed by a runner and currently executing."),
    succeeded: t("Completed successfully with no reported errors."),
    failed: t("Finished with an error. Inspect the failure output below."),
    cancelled: t("Stopped before completion."),
  };
  return descriptions[status];
};

const getJobQueueDiagnosticMessage = (stats: JobQueueStats, t: (text: string) => string): string => {
  if (stats.failed > 0) {
    return t("Start with failed jobs: read last_error, open the related pipeline job trace when available, and verify whether artifacts were expected but missing.");
  }
  if (stats.pending > 0 && stats.running === 0) {
    return t("Queued jobs with no active workers usually indicate runner capacity, heartbeat, or tag-matching problems.");
  }
  if (stats.running > 0) {
    return t("Running jobs may not have flushed output yet. Reload after completion or inspect the pipeline job trace for live diagnostics.");
  }
  return t("Use this queue as the low-level CI control plane: result text, last_error, attempts, worker assignment, trace, and artifacts together explain job outcomes.");
};

const getJobTraceArtifactGuidance = (job: RepositoryJobView, t: (text: string) => string): string => {
  if (job.status === "failed") {
    return t("For failures, compare last_error with the pipeline trace and inspect artifacts for test reports, screenshots, or partial build output.");
  }
  if (job.status === "pending") {
    return t("Trace and artifacts are not expected until a runner claims the job and starts the script.");
  }
  if (job.status === "running") {
    return t("Trace output may still be streaming. If artifacts are missing, wait until the runner completes upload.");
  }
  if (job.status === "succeeded") {
    return t("A successful job can still have empty output. Use artifacts as the durable evidence when the script uploaded files.");
  }
  return t("Cancelled jobs may have partial trace output but should not be expected to produce complete artifacts.");
};

const getJobOutputEmptyMessage = (status: RepositoryJobStatus, t: (text: string) => string): string => {
  if (status === "pending") {
    return t("No log is available because the job is still queued and has not been claimed by a runner.");
  }
  if (status === "running") {
    return t("The runner has not stored output yet. Reload after the next trace flush or after completion.");
  }
  if (status === "failed") {
    return t("The job failed without stored result output. Use last_error, attempt count, worker assignment, and the pipeline trace to diagnose it.");
  }
  if (status === "succeeded") {
    return t("The job succeeded without a stored result payload. This can be normal for jobs that only update status or upload artifacts.");
  }
  return t("The job was cancelled before a durable result payload was recorded.");
};

const JobStatusBadge = ({ status, t }: { status: RepositoryJobStatus; t: (text: string) => string }) => {
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

const normalizeJob = (raw: RawProjectJob): RepositoryJobView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  kind: normalizeString(raw.kind ?? raw.Kind) || "noop",
  status: normalizeStatus(raw.status ?? raw.Status),
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
});

const resolveJobList = (payload: unknown): RawProjectJob[] => {
  if (Array.isArray(payload)) {
    return resolveRecordArray(payload);
  }
  return isRecord(payload) ? resolveRecordArray(payload.body ?? payload.Body) : [];
};

const normalizeStatus = (value: unknown): RepositoryJobStatus => {
  const normalized = normalizeString(value);
  if (normalized === "canceled") {
    return "cancelled";
  }
  if (normalized === "pending" || normalized === "running" || normalized === "succeeded" || normalized === "failed" || normalized === "cancelled") {
    return normalized;
  }
  return "pending";
};
