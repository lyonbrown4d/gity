import { useMemo, useState } from "react";
import { FileCode2, GitBranch } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { CiPipelineDetail } from "@/lib/generated/model/ciPipelineDetail";
import { parseConfig } from "./config-model";
import { PipelineGraph } from "./pipeline-graph";
import { CIStatus, elapsed } from "./status";
interface Props {
  detail: CiPipelineDetail;
  t: (key: string) => string;
  onOpenJob: (jobId: string) => void;
}
export const PipelineRunDetail = ({ detail, t, onOpenJob }: Props) => {
  const [tab, setTab] = useState("overview");
  const { pipeline, jobs } = detail;
  const parsed = useMemo(() => parseConfig(pipeline.config_content), [pipeline.config_content]);
  const statuses = useMemo(() => Object.fromEntries(jobs.map((job) => [job.pipeline_job.name, job.status])), [jobs]);
  const failed = jobs.filter((job) => job.status === "failed");
  const openJob = (name: string) => {
    const selected = jobs.find((job) => job.pipeline_job.name === name);
    if (selected) onOpenJob(selected.project_job.id);
  };
  return (
    <section className="space-y-5">
      <header>
        <div className="flex flex-wrap items-center gap-3">
          <h3 className="text-2xl font-semibold tracking-tight">
            {t("Pipeline")} #{pipeline.iid}
          </h3>
          <CIStatus status={pipeline.status} t={t} />
        </div>
        <p className="mt-2 text-base">{pipeline.name}</p>
        <div className="mt-3 flex flex-wrap gap-5 text-sm text-muted-foreground">
          <span className="flex items-center gap-1">
            <GitBranch className="size-4" />
            {pipeline.ref_name}
          </span>
          <code className="rounded bg-muted px-2">{pipeline.commit_sha.slice(0, 12)}</code>
          <span>{t(pipeline.source)}</span>
          <span>{pipeline.created_at ? new Date(pipeline.created_at).toLocaleString() : "—"}</span>
          <span>
            {t("Duration")}: {elapsed(pipeline.started_at, pipeline.finished_at)}
          </span>
        </div>
      </header>
      <div className="flex gap-2 border-b border-border pb-2">
        <Button variant={tab === "overview" ? "secondary" : "ghost"} onClick={() => setTab("overview")}>
          {t("Overview")}
        </Button>
        <Button variant={tab === "configuration" ? "secondary" : "ghost"} onClick={() => setTab("configuration")}>
          <FileCode2 className="mr-1 size-4" />
          {t("Configuration")}
        </Button>
      </div>
      {tab === "configuration" ? (
        <div className="rounded-lg border border-border bg-card p-4">
          <p className="mb-3 text-xs text-muted-foreground">
            {t("Configuration pinned to this run")}: {pipeline.commit_sha}
          </p>
          <pre className="max-h-[600px] overflow-auto whitespace-pre text-xs">{pipeline.config_content}</pre>
        </div>
      ) : (
        <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_260px]">
          <div className="min-w-0 space-y-4">
            {failed.length > 0 && (
              <div
                role="status"
                className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-destructive/25 bg-destructive/10 px-4 py-2 text-sm text-destructive"
              >
                <span>
                  {t("Failed jobs")}: {failed.map((job) => job.pipeline_job.name).join(", ")}
                </span>
                <Button variant="ghost" size="sm" onClick={() => openJob(failed[0].pipeline_job.name)}>
                  {t("View job logs")}
                </Button>
              </div>
            )}
            <div className="overflow-hidden rounded-lg border border-border bg-card">
              {parsed.config ? (
                <PipelineGraph config={parsed.config} statuses={statuses} readonly t={t} onSelect={openJob} />
              ) : (
                <p role="alert" className="p-5 text-sm text-muted-foreground">
                  {t("The stored configuration cannot be displayed as a graph.")}
                </p>
              )}
            </div>
            <div className="overflow-x-auto rounded-lg border border-border bg-card">
              <table className="w-full text-left text-sm">
                <thead className="bg-muted/50 text-xs text-muted-foreground">
                  <tr>
                    {["Task name", "Stage", "Status", "Duration", "Runner", "Attempt", "Actions"].map((label) => (
                      <th className="px-3 py-3 font-medium" key={label}>
                        {t(label)}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {jobs.map((job) => (
                    <tr key={job.project_job.id} className="border-t border-border hover:bg-muted/25">
                      <td className="px-3 py-3 font-medium">{job.pipeline_job.name}</td>
                      <td className="px-3">{job.pipeline_job.stage}</td>
                      <td className="px-3">
                        <CIStatus status={job.status} t={t} />
                      </td>
                      <td className="px-3 whitespace-nowrap">
                        {elapsed(job.project_job.started_at, job.project_job.finished_at)}
                      </td>
                      <td className="px-3">{job.project_job.locked_by || "—"}</td>
                      <td className="px-3">{job.project_job.attempts}</td>
                      <td className="px-3">
                        <Button variant="link" size="sm" onClick={() => openJob(job.pipeline_job.name)}>
                          {t("View logs")}
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {!jobs.length && <p className="p-5 text-sm text-muted-foreground">{t("No jobs in this pipeline.")}</p>}
            </div>
          </div>
          <aside className="space-y-4 rounded-lg border border-border bg-card p-5">
            <h4 className="font-semibold">{t("Run result")}</h4>
            <dl className="space-y-4 text-sm">
              {[
                ["Pipeline", `#${pipeline.iid}`],
                ["Source", t(pipeline.source)],
                ["Branch", pipeline.ref_name],
                ["Commit", pipeline.commit_sha.slice(0, 12)],
                ["Duration", elapsed(pipeline.started_at, pipeline.finished_at)],
              ].map(([label, value]) => (
                <div className="flex justify-between gap-3" key={label}>
                  <dt className="text-muted-foreground">{t(label)}</dt>
                  <dd className="break-all text-right">{value}</dd>
                </div>
              ))}
            </dl>
            {failed.map((job) => (
              <div className="border-t border-border pt-4" key={job.project_job.id}>
                <button
                  className="text-sm font-semibold text-destructive"
                  onClick={() => openJob(job.pipeline_job.name)}
                >
                  {job.pipeline_job.name}
                </button>
                {job.project_job.last_error && (
                  <p className="mt-2 text-xs text-destructive">{job.project_job.last_error}</p>
                )}
                <pre className="mt-3 overflow-auto rounded bg-muted/50 p-2 text-xs">{job.script.join("\n")}</pre>
              </div>
            ))}
          </aside>
        </div>
      )}
    </section>
  );
};
