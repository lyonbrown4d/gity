import { useState } from "react";
import { useCustom, useCustomMutation, useDataProvider } from "@refinedev/core";
import { Download, RefreshCw, Repeat2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { FormDialog } from "@/components/common/form-dialog";
import type { CiJob } from "@/lib/generated/model/ciJob";
import type { CiJobArtifact } from "@/lib/generated/model/ciJobArtifact";
import type { CiJobArtifactDownload } from "@/lib/generated/model/ciJobArtifactDownload";
import { extractErrorMessage } from "../issues-utils";
import { CIStatus, isTerminal, saveBlob } from "./status";
import { useLogDownload } from "./use-log-download";
import { useJobTrace } from "./use-job-trace";
import { LogSegmentControls } from "./log-segment-controls";
import { JobLogViewer } from "./job-log-viewer";
interface Props {
  repoId: string;
  job: CiJob;
  name?: string;
  canWrite: boolean;
  t: (key: string) => string;
  onChanged?: () => void;
}
export const JobDetail = ({ repoId, job: initialJob, name, canWrite, t, onChanged }: Props) => {
  const [polling, setPolling] = useState(true);
  const [tab, setTab] = useState("logs");
  const [error, setError] = useState<string | null>(null);
  const [confirmCancel, setConfirmCancel] = useState(false);
  const [downloading, setDownloading] = useState<string | null>(null);
  const provider = useDataProvider();
  const trace = useJobTrace(repoId, initialJob, polling);
  const artifacts = useCustom<CiJobArtifact[]>({
    url: `/projects/${repoId}/jobs/${initialJob.id}/artifacts`,
    method: "get",
    queryOptions: {
      refetchOnWindowFocus: false,
      refetchInterval: polling && !isTerminal(initialJob.status) ? 5000 : false,
    },
  });
  const { mutateAsync, mutation } = useCustomMutation();
  const job = trace.data?.job ?? initialJob;
  const log = trace.data?.log;
  const logDownload = useLogDownload(repoId, initialJob.id, trace.data?.attempt);
  const action = async (operation: "retry" | "cancel") => {
    if (mutation.isPending) return;
    try {
      await mutateAsync({ url: `/projects/${repoId}/jobs/${job.id}/${operation}`, method: "post", values: {} });
      if (operation === "retry") trace.selectAttempt(null);
      await trace.query.refetch();
      onChanged?.();
      setConfirmCancel(false);
      setError(null);
    } catch (failure) {
      setError(extractErrorMessage(failure));
    }
  };
  const download = async (artifact: CiJobArtifact) => {
    if (downloading) return;
    setDownloading(artifact.id);
    try {
      const response = await provider().custom?.<CiJobArtifactDownload>({
        url: `/projects/${repoId}/jobs/${job.id}/artifacts/${artifact.id}`,
        method: "get",
      });
      if (!response) throw new Error(t("Artifact download is not available."));
      const content = Uint8Array.from(atob(response.data.content_base64), (char) => char.charCodeAt(0));
      saveBlob(
        new Blob([content], { type: response.data.content_type || "application/octet-stream" }),
        response.data.file_name,
      );
    } catch (failure) {
      setError(extractErrorMessage(failure));
    } finally {
      setDownloading(null);
    }
  };
  return (
    <section className="min-w-0 space-y-4">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-3">
            <h3 className="text-2xl font-semibold">{name || `${t("Job")} #${job.id}`}</h3>
            <CIStatus status={job.status} t={t} />
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            {t("Attempt")} {log?.attempt ?? job.attempts} · {t("Runner")}: {job.locked_by || "—"} ·{" "}
            {log?.duration_millis != null ? `${(log.duration_millis / 1000).toFixed(1)}s` : "—"}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              void trace.query.refetch();
              void artifacts.query.refetch();
            }}
          >
            <RefreshCw className="mr-1 size-4" />
            {t("Reload")}
          </Button>
          <Button
            size="sm"
            disabled={!canWrite || mutation.isPending}
            variant={isTerminal(job.status) ? "default" : "outline"}
            onClick={() => (isTerminal(job.status) ? void action("retry") : setConfirmCancel(true))}
          >
            <Repeat2 className="mr-1 size-4" />
            {t(isTerminal(job.status) ? "Retry job" : "Cancel job")}
          </Button>
        </div>
      </header>
      {job.last_error && (
        <p
          role="alert"
          className="rounded-md border border-destructive/25 bg-destructive/10 p-3 text-sm text-destructive"
        >
          {job.last_error}
        </p>
      )}
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
      <div className="flex flex-wrap items-center gap-3 border-b border-border pb-2">
        <Button variant={tab === "logs" ? "secondary" : "ghost"} onClick={() => setTab("logs")}>
          {t("Logs")}
        </Button>
        <Button variant={tab === "artifacts" ? "secondary" : "ghost"} onClick={() => setTab("artifacts")}>
          {t("Artifacts")} ({artifacts.result.data?.length ?? 0})
        </Button>
        <label className="ml-auto flex items-center gap-2 text-xs">
          <input type="checkbox" checked={polling} onChange={(event) => setPolling(event.target.checked)} />
          {t("Refresh while running")}
        </label>
      </div>
      {trace.query.error && (
        <p role="alert" className="rounded-md border border-destructive/25 p-3 text-sm text-destructive">
          {t("Log refresh failed; showing the last received output.")} {extractErrorMessage(trace.query.error)}
        </p>
      )}
      {tab === "logs" ? (
        <>
          <div className="flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground">
            <label>
              {t("Attempt")}{" "}
              <select
                className="ml-2 rounded border border-input bg-background p-2"
                aria-label={t("Attempt")}
                value={trace.attempt ?? "latest"}
                onChange={(event) =>
                  trace.selectAttempt(event.target.value === "latest" ? null : Number(event.target.value))
                }
              >
                <option value="latest">{t("Latest attempt")}</option>
                {trace.data?.attempt_summaries?.map((item) => (
                  <option key={item.attempt} value={item.attempt}>
                    {item.attempt} · {t("Exit code")} {item.exit_code}
                  </option>
                ))}
              </select>
            </label>
            <span>
              {t("Last updated")}:{" "}
              {trace.query.dataUpdatedAt ? new Date(trace.query.dataUpdatedAt).toLocaleTimeString() : "—"}
            </span>
            {log && isTerminal(job.status) && (
              <span>
                {t("Exit code")}: {log.exit_code}
              </span>
            )}
          </div>
          <LogSegmentControls
            offset={trace.data?.offset ?? 0}
            nextOffset={trace.data?.next_offset ?? 0}
            total={trace.data?.total_bytes ?? 0}
            hasMore={trace.data?.has_more ?? false}
            pending={trace.query.isFetching}
            onBeginning={trace.beginning}
            onPrevious={trace.previous}
            onNext={trace.next}
            onLatest={trace.latest}
            t={t}
          />
          {trace.query.isLoading ? (
            <p role="status">{t("Loading logs...")}</p>
          ) : (
            <JobLogViewer
              key={`${job.id}:${trace.data?.attempt ?? "latest"}:${trace.data?.offset ?? 0}`}
              output={trace.data?.output ?? ""}
              jobId={job.id}
              failed={job.status === "failed"}
              following={trace.following}
              onFollowingChange={trace.setFollowing}
              onDownloadFull={logDownload.supported ? () => void logDownload.start() : undefined}
              downloading={logDownload.pending}
              t={t}
            />
          )}
          <p className="text-xs text-muted-foreground">{t("Search and copy apply to the loaded segment.")}</p>
          {logDownload.pending && (
            <div role="status" className="flex items-center gap-3 text-xs text-muted-foreground">
              <span>
                {t("Downloading logs")}:{" "}
                {logDownload.progress
                  ? `${logDownload.progress.received.toLocaleString()} / ${logDownload.progress.total.toLocaleString()} B`
                  : "…"}
              </span>
              <Button size="sm" variant="outline" onClick={logDownload.cancel}>
                {t("Cancel download")}
              </Button>
            </div>
          )}
          {logDownload.error && (
            <p role="alert" className="text-sm text-destructive">
              {t(logDownload.error)}
            </p>
          )}
          {Boolean(trace.data?.truncated) && (
            <p className="text-xs text-muted-foreground">{t("The runner truncated this output.")}</p>
          )}
        </>
      ) : (
        <div className="rounded-lg border border-border bg-card">
          {artifacts.query.isLoading ? (
            <p className="p-5">{t("Loading artifacts...")}</p>
          ) : artifacts.query.error ? (
            <p role="alert" className="p-5 text-destructive">
              {extractErrorMessage(artifacts.query.error)}
            </p>
          ) : !artifacts.result.data?.length ? (
            <p className="p-5 text-sm text-muted-foreground">{t("No artifacts uploaded.")}</p>
          ) : (
            artifacts.result.data.map((artifact) => (
              <div
                className="flex items-center justify-between gap-3 border-b border-border p-4 last:border-0"
                key={artifact.id}
              >
                <div>
                  <p className="text-sm font-medium">{artifact.file_name}</p>
                  <p className="text-xs text-muted-foreground">{artifact.byte_size.toLocaleString()} B</p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={Boolean(downloading)}
                  onClick={() => void download(artifact)}
                >
                  <Download className="mr-1 size-4" />
                  {t("Download")}
                </Button>
              </div>
            ))
          )}
        </div>
      )}
      <FormDialog
        open={confirmCancel}
        onClose={() => {
          if (!mutation.isPending) setConfirmCancel(false);
        }}
        title={t("Cancel job")}
      >
        <p className="mb-4 text-sm">{t("Cancel this job and stop its current execution?")}</p>
        <Button variant="destructive" disabled={mutation.isPending} onClick={() => void action("cancel")}>
          {t("Cancel job")}
        </Button>
      </FormDialog>
    </section>
  );
};
