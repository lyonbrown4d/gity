import { Link, useParams } from "react-router";
import { useCustom } from "@refinedev/core";
import { Button } from "@/components/ui/button";
import type { CiJob } from "@/lib/generated/model/ciJob";
import type { CiPipelineDetail } from "@/lib/generated/model/ciPipelineDetail";
import { extractErrorMessage } from "../issues-utils";
import { JobDetail } from "./job-detail";
import { CIStatus, isTerminal } from "./status";
interface Props {
  repoId: string;
  jobId: string;
  base: string;
  canWrite: boolean;
  t: (key: string) => string;
}
export const JobRouteDetail = ({ repoId, jobId, base, canWrite, t }: Props) => {
  const { pipelineId } = useParams<{ pipelineId?: string }>();
  const job = useCustom<CiJob>({
    url: `/projects/${repoId}/jobs/${jobId}`,
    method: "get",
    queryOptions: {
      refetchOnWindowFocus: false,
      refetchInterval: (query) => (isTerminal(query.state.data?.data?.status ?? "running") ? false : 5000),
    },
  });
  const pipeline = useCustom<CiPipelineDetail>({
    url: `/projects/${repoId}/pipelines/${pipelineId ?? "0"}`,
    method: "get",
    queryOptions: {
      enabled: Boolean(pipelineId),
      refetchOnWindowFocus: false,
      refetchInterval: pipelineId ? 5000 : false,
    },
  });
  const current = job.result.data?.id === jobId ? job.result.data : null;
  const selected = pipeline.result.data?.jobs?.find((item) => item.project_job.id === jobId);
  const context = selected ? pipeline.result.data : null;
  const changed = () => {
    void job.query.refetch();
    if (pipelineId) void pipeline.query.refetch();
  };
  return (
    <section className="space-y-5">
      <Button size="sm" variant="outline" asChild>
        <Link to={context ? `${base}/pipelines/${context.pipeline.id}` : `${base}/jobs`}>
          {t(context ? "Back to pipeline" : "Back to jobs")}
          {context ? ` #${context.pipeline.iid}` : ""}
        </Link>
      </Button>
      {job.query.error && (
        <p role="alert" className="text-sm text-destructive">
          {extractErrorMessage(job.query.error)}
        </p>
      )}
      {pipeline.query.error && (
        <p role="alert" className="text-xs text-muted-foreground">
          {extractErrorMessage(pipeline.query.error)}
        </p>
      )}
      {job.query.isLoading && <p role="status">{t("Loading job...")}</p>}
      {current && (
        <div className="flex flex-col gap-6 xl:flex-row">
          {context && (
            <nav
              className="max-h-[600px] shrink-0 space-y-1 overflow-auto rounded-lg border border-border bg-card p-2 xl:w-52"
              aria-label={t("Pipeline jobs")}
            >
              {context.jobs.map((item) => (
                <Link
                  key={item.project_job.id}
                  to={`${base}/pipelines/${context.pipeline.id}/jobs/${item.project_job.id}`}
                  aria-current={item.project_job.id === jobId ? "page" : undefined}
                  className={`flex flex-wrap items-center justify-between gap-2 rounded-md p-3 text-sm hover:bg-muted ${item.project_job.id === jobId ? "bg-primary/10" : ""}`}
                >
                  <span className="font-medium">{item.pipeline_job.name}</span>
                  <CIStatus status={item.status} t={t} />
                </Link>
              ))}
            </nav>
          )}
          <div className="min-w-0 flex-1">
            <JobDetail
              key={current.id}
              repoId={repoId}
              job={current}
              name={selected?.pipeline_job.name}
              canWrite={canWrite}
              t={t}
              onChanged={changed}
            />
          </div>
        </div>
      )}
    </section>
  );
};
