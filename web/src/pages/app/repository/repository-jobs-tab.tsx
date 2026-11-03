import { useState } from "react";
import { useCustom } from "@refinedev/core";
import { Link, useParams } from "react-router";
import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { CiJob } from "@/lib/generated/model/ciJob";
import type { RepositoryPermissions } from "./repository-permissions";
import { extractErrorMessage } from "./issues-utils";
import { CIStatus, elapsed } from "./ci/status";
import { JobRouteDetail } from "./ci/job-route-detail";
import { EnqueueJobDialog } from "./ci/enqueue-job-dialog";
interface Props {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (key: string) => string;
  onError: (message: string | null) => void;
}
export const RepositoryJobsTab = ({ repoId, permissions, t }: Props): JSX.Element => {
  const [createOpen, setCreateOpen] = useState(false);
  const { organizationId, jobId } = useParams<{ organizationId: string; jobId?: string }>();
  const base = `/app/projects/${encodeURIComponent(organizationId ?? "")}/${encodeURIComponent(repoId)}`;
  const jobs = useCustom<CiJob[]>({
    url: `/projects/${repoId}/jobs`,
    method: "get",
    queryOptions: { enabled: !jobId, refetchOnWindowFocus: false, refetchInterval: jobId ? false : 5000 },
  });
  if (jobId) return <JobRouteDetail repoId={repoId} jobId={jobId} base={base} canWrite={permissions.jobWrite} t={t} />;
  return (
    <section className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">{t("Jobs")}</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            {t("Inspect job execution, logs, attempts and artifacts.")}
          </p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" disabled={!permissions.jobWrite} onClick={() => setCreateOpen(true)}>
            {t("New job")}
          </Button>
          <Button size="sm" variant="outline" onClick={() => void jobs.query.refetch()}>
            <RefreshCw className="mr-1 size-4" />
            {t("Reload")}
          </Button>
        </div>
      </header>
      {jobs.query.error && (
        <p role="alert" className="text-sm text-destructive">
          {extractErrorMessage(jobs.query.error)}
        </p>
      )}
      {jobs.query.isLoading && <p role="status">{t("Loading jobs...")}</p>}
      {!jobs.query.isLoading && !jobs.query.error && (
        <div className="overflow-x-auto rounded-lg border border-border bg-card">
          <table className="w-full text-left text-sm">
            <thead className="bg-muted/50 text-xs text-muted-foreground">
              <tr>
                {["Job", "Kind", "Status", "Runner", "Attempt", "Duration", "Actions"].map((label) => (
                  <th key={label} className="px-4 py-3 font-medium">
                    {t(label)}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {jobs.result.data?.map((job) => (
                <tr key={job.id} className="border-t border-border hover:bg-muted/20">
                  <td className="px-4 py-3 font-medium">#{job.id}</td>
                  <td className="px-4">{job.kind}</td>
                  <td className="px-4">
                    <CIStatus status={job.status} t={t} />
                  </td>
                  <td className="px-4">{job.locked_by || "—"}</td>
                  <td className="px-4">
                    {job.attempts} / {job.max_attempts}
                  </td>
                  <td className="px-4 whitespace-nowrap">{elapsed(job.started_at, job.finished_at)}</td>
                  <td className="px-4">
                    <Button size="sm" variant="link" asChild>
                      <Link to={`${base}/jobs/${job.id}`}>{t("View logs")}</Link>
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {!jobs.result.data?.length && (
            <p className="p-8 text-center text-sm text-muted-foreground">{t("No jobs yet.")}</p>
          )}
        </div>
      )}
      <EnqueueJobDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        repoId={repoId}
        onCreated={() => void jobs.query.refetch()}
        t={t}
      />
    </section>
  );
};
