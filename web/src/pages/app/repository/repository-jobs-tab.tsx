import { useEffect, useMemo, useState } from "react";
import { Ban, CheckCircle2, Clock3, Loader2, Play, RefreshCw, XCircle } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Textarea } from "@/components/ui/textarea";
import type { RepositoryJobStatus, RepositoryJobView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";

interface RepositoryJobsTabProps {
  repoId: string;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

type RawProjectJob = Record<string, unknown>;

const terminalStatuses: RepositoryJobStatus[] = ["succeeded", "failed", "cancelled"];

export const RepositoryJobsTab = ({ repoId, t, onError }: RepositoryJobsTabProps): JSX.Element => {
  const jobsQuery = useCustom<RawProjectJob[]>({
    url: `/projects/${repoId}/jobs`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: enqueueJob, isLoading: isEnqueueing } = useCustomMutation<RawProjectJob>();
  const { mutateAsync: cancelJob, isLoading: isCancelling } = useCustomMutation<RawProjectJob>();
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [kind, setKind] = useState("noop");
  const [payload, setPayload] = useState("");
  const [maxAttempts, setMaxAttempts] = useState("3");

  const jobs = useMemo(
    () => resolveJobList(jobsQuery.data?.data).map(normalizeJob),
    [jobsQuery.data?.data],
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
  const isLoadingJobs = jobsQuery.isFetching && !jobsQuery.data;

  const loadJobs = async () => {
    const result = await jobsQuery.refetch();
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
    if (!jobsQuery.error) {
      return;
    }
    onError(extractErrorMessage(jobsQuery.error));
  }, [jobsQuery.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Jobs")}</CardTitle>
        <CardDescription>{t("Inspect and enqueue project background jobs.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-4">
          <JobStat label={t("Pending")} value={stats.pending} tone="amber" />
          <JobStat label={t("Running")} value={stats.running} tone="blue" />
          <JobStat label={t("Succeeded")} value={stats.completed} tone="emerald" />
          <JobStat label={t("Failed")} value={stats.failed} tone="rose" />
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isComposerOpen ? "secondary" : "outline"}
            onClick={() => setComposerOpen((current) => !current)}
          >
            <Play className="size-4" />
            {isComposerOpen ? t("Hide new job form") : t("New job")}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadJobs()}>
            <RefreshCw className="size-4" />
            {t("Reload")}
          </Button>
        </div>

        {isComposerOpen ? (
          <form className="space-y-3 rounded-md border p-3" onSubmit={submitEnqueueJob}>
            <div className="grid gap-3 md:grid-cols-[1fr_160px]">
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="job-kind">
                  {t("Job kind")}
                </label>
                <Input
                  id="job-kind"
                  value={kind}
                  onChange={(event) => setKind(event.target.value)}
                  placeholder="noop"
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="job-max-attempts">
                  {t("Max attempts")}
                </label>
                <Input
                  id="job-max-attempts"
                  type="number"
                  min={1}
                  value={maxAttempts}
                  onChange={(event) => setMaxAttempts(event.target.value)}
                />
              </div>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium text-muted-foreground" htmlFor="job-payload">
                {t("Payload JSON")}
              </label>
              <Textarea
                id="job-payload"
                className="min-h-24"
                value={payload}
                onChange={(event) => setPayload(event.target.value)}
                placeholder='{"reason":"manual"}'
              />
            </div>
            <div className="flex justify-end">
              <Button type="submit" disabled={isEnqueueing}>
                {isEnqueueing ? t("Enqueueing job...") : t("Enqueue job")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="space-y-2 rounded-md border p-2">
          {isLoadingJobs ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading jobs...")}</p> : null}
          {!isLoadingJobs && jobs.length === 0 ? (
            <p className="px-2 py-2 text-sm text-muted-foreground">{t("No jobs found.")}</p>
          ) : null}
          {jobs.map((job) => (
            <div key={job.id} className="rounded-md border p-3">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="min-w-0 space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="font-medium">
                      #{job.id} {job.kind}
                    </p>
                    <JobStatusBadge status={job.status} t={t} />
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {t("Attempts")}: {job.attempts}/{job.max_attempts}
                    {job.locked_by ? ` · ${t("Worker")}: ${job.locked_by}` : ""}
                    {job.updated_at ? ` · ${formatRelativeTime(job.updated_at)}` : ""}
                  </p>
                </div>
                {!terminalStatuses.includes(job.status) ? (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={isCancelling}
                    onClick={() => void submitCancelJob(job)}
                  >
                    <Ban className="size-4" />
                    {t("Cancel job")}
                  </Button>
                ) : null}
              </div>
              {job.last_error ? (
                <Alert variant="destructive" className="mt-2 px-2 py-1 text-xs">
                  <AlertDescription className="text-xs leading-5">{job.last_error}</AlertDescription>
                </Alert>
              ) : null}
              {job.result ? (
                <pre className="mt-2 overflow-auto rounded-md bg-muted p-2 text-xs">{job.result}</pre>
              ) : null}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

const JobStat = ({ label, value, tone }: { label: string; value: number; tone: "amber" | "blue" | "emerald" | "rose" }) => {
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

const JobStatusBadge = ({ status, t }: { status: RepositoryJobStatus; t: (text: string) => string }) => {
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

const isRecord = (value: unknown): value is RawProjectJob =>
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

const normalizeStatus = (value: unknown): RepositoryJobStatus => {
  const normalized = normalizeString(value);
  if (normalized === "pending" || normalized === "running" || normalized === "succeeded" || normalized === "failed" || normalized === "cancelled") {
    return normalized;
  }
  return "pending";
};
