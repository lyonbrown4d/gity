import { lazy, Suspense, useState } from "react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { useLocation, useNavigate, useParams } from "react-router";
import { FileCode2, Play, RefreshCw, Repeat2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { FormDialog } from "@/components/common/form-dialog";
import type { RepositoryBranchView } from "@/pages/types";
import type { CiPipeline } from "@/lib/generated/model/ciPipeline";
import type { CiPipelineDetail } from "@/lib/generated/model/ciPipelineDetail";
import type { RepositoryPermissions } from "./repository-permissions";
import { extractErrorMessage } from "./issues-utils";
import { isTerminal } from "./ci/status";
import { RecentRuns } from "./ci/recent-runs";
import { PipelineRunDetail } from "./ci/pipeline-run-detail";
const CIEditor = lazy(() => import("./ci/ci-editor").then((module) => ({ default: module.CIEditor })));
interface Props {
  repoId: string;
  defaultBranch: string;
  permissions: RepositoryPermissions;
  t: (key: string) => string;
  onError: (message: string | null) => void;
}
export const RepositoryPipelinesTab = ({ repoId, defaultBranch, permissions, t, onError }: Props): JSX.Element => {
  const { organizationId, pipelineId } = useParams<{ organizationId: string; pipelineId?: string }>();
  const navigate = useNavigate();
  const editing = useLocation().pathname.endsWith("/pipelines/editor");
  const base = `/app/projects/${encodeURIComponent(organizationId ?? "")}/${encodeURIComponent(repoId)}`;
  const [createOpen, setCreateOpen] = useState(false);
  const [cancelOpen, setCancelOpen] = useState(false);
  const [branch, setBranch] = useState(defaultBranch || "main");
  const [error, setError] = useState<string | null>(null);
  const pipelines = useCustom<CiPipeline[]>({
    url: `/projects/${repoId}/pipelines`,
    method: "get",
    queryOptions: { refetchOnWindowFocus: false, refetchInterval: editing ? false : 5000 },
  });
  const selected = pipelineId ?? pipelines.result.data?.[0]?.id;
  const detail = useCustom<CiPipelineDetail>({
    url: `/projects/${repoId}/pipelines/${selected ?? "0"}`,
    method: "get",
    queryOptions: {
      enabled: Boolean(selected) && !editing,
      refetchOnWindowFocus: false,
      refetchInterval: !editing ? 5000 : false,
    },
  });
  const branches = useCustom<RepositoryBranchView[]>({
    url: `/projects/${repoId}/repository/branches`,
    method: "get",
    queryOptions: { enabled: createOpen, refetchOnWindowFocus: false },
  });
  const { mutateAsync, mutation } = useCustomMutation<CiPipelineDetail>();
  const visible = selected && detail.result.data?.pipeline?.id === selected ? detail.result.data : null;
  const branchRows = Array.isArray(branches.result.data) ? branches.result.data : [];
  const reload = () => {
    void pipelines.query.refetch();
    void detail.query.refetch();
  };
  const setEditor = (open: boolean) => navigate(`${base}/pipelines${open ? "/editor" : ""}`);
  const action = async (operation: "retry" | "cancel" | "refresh") => {
    if (!selected || mutation.isPending) return;
    try {
      await mutateAsync({ url: `/projects/${repoId}/pipelines/${selected}/${operation}`, method: "post", values: {} });
      reload();
      setCancelOpen(false);
      setError(null);
      onError(null);
    } catch (failure) {
      setError(extractErrorMessage(failure));
    }
  };
  const create = async (event: React.FormEvent) => {
    event.preventDefault();
    const selectedBranch = branchRows.find((item) => item.name === branch);
    if (mutation.isPending || !selectedBranch?.last_commit_sha) return;
    try {
      const response = await mutateAsync({
        url: `/projects/${repoId}/pipelines`,
        method: "post",
        values: {
          source: "web",
          ref_name: branch,
          commit_sha: selectedBranch.last_commit_sha,
          config_source: ".gity-ci.yml",
          config_content: "",
        },
      });
      navigate(`${base}/pipelines/${response.data.pipeline.id}`);
      setCreateOpen(false);
      setError(null);
      reload();
    } catch (failure) {
      setError(extractErrorMessage(failure));
    }
  };
  if (editing)
    return (
      <Suspense fallback={<p role="status">{t("Loading editor...")}</p>}>
        <CIEditor
          repoId={repoId}
          defaultBranch={defaultBranch}
          permissions={permissions}
          t={t}
          onClose={() => setEditor(false)}
        />
      </Suspense>
    );
  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">{t("Pipelines")}</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            {t("Track CI pipelines triggered by pushes and merge requests.")}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => setEditor(true)}>
            <FileCode2 className="mr-2 size-4" />
            {t("Edit configuration")}
          </Button>
          <Button
            disabled={!permissions.ciWrite}
            onClick={() => {
              setError(null);
              setCreateOpen(true);
            }}
          >
            <Play className="mr-2 size-4" />
            {t("Run pipeline")}
          </Button>
          <Button variant="ghost" size="icon" aria-label={t("Reload")} onClick={reload}>
            <RefreshCw className="size-4" />
          </Button>
        </div>
      </header>
      {(error || pipelines.query.error || detail.query.error) && (
        <p role="alert" className="rounded-md border border-destructive/25 p-3 text-sm text-destructive">
          {error || extractErrorMessage(pipelines.query.error ?? detail.query.error)}
        </p>
      )}
      {pipelines.query.isLoading && <p role="status">{t("Loading pipelines...")}</p>}
      {!pipelines.query.isLoading && !pipelines.query.error && !pipelines.result.data?.length && (
        <div className="rounded-lg border border-border bg-card p-10 text-center">
          <h3 className="font-semibold">{t("No pipelines yet")}</h3>
          <p className="mt-2 text-sm text-muted-foreground">
            {t("Commit .gity-ci.yml to your repository, then run a pipeline.")}
          </p>
        </div>
      )}
      {visible && (
        <>
          <div className="flex justify-end gap-2">
            <span className="mr-auto self-center text-xs text-muted-foreground">
              {t("Last updated")}:{" "}
              {detail.query.dataUpdatedAt ? new Date(detail.query.dataUpdatedAt).toLocaleTimeString() : "—"}
            </span>
            <Button
              variant="outline"
              disabled={!permissions.ciWrite || mutation.isPending}
              onClick={() => void action("refresh")}
            >
              {t("Refresh state")}
            </Button>
            <Button
              variant="outline"
              disabled={!permissions.ciWrite || mutation.isPending}
              onClick={() => (isTerminal(visible.pipeline.status) ? void action("retry") : setCancelOpen(true))}
            >
              <Repeat2 className="mr-1 size-4" />
              {t(isTerminal(visible.pipeline.status) ? "Retry pipeline" : "Cancel pipeline")}
            </Button>
          </div>
          <PipelineRunDetail
            key={visible.pipeline.id}
            detail={visible}
            t={t}
            onOpenJob={(jobId) => navigate(`${base}/pipelines/${visible.pipeline.id}/jobs/${jobId}`)}
          />
        </>
      )}
      {detail.query.isLoading && selected && <p role="status">{t("Loading pipeline...")}</p>}
      {Boolean(pipelines.result.data?.length) && (
        <RecentRuns
          pipelines={pipelines.result.data ?? []}
          selected={selected}
          onSelect={(id) => navigate(`${base}/pipelines/${id}`)}
          t={t}
        />
      )}
      <FormDialog
        open={createOpen}
        onClose={() => {
          if (!mutation.isPending) setCreateOpen(false);
        }}
        title={t("Run pipeline")}
      >
        <form className="space-y-4" onSubmit={create}>
          <p className="text-sm text-muted-foreground">
            {t("Run the committed .gity-ci.yml at the selected revision.")}
          </p>
          <label className="block space-y-2 text-sm">
            {t("Branch")}
            <select
              className="h-10 w-full rounded-md border border-input bg-background px-3"
              value={branch}
              onChange={(event) => setBranch(event.target.value)}
            >
              {branchRows.map((item) => (
                <option key={item.name}>{item.name}</option>
              ))}
            </select>
          </label>
          <p className="break-all font-mono text-xs">
            {branchRows.find((item) => item.name === branch)?.last_commit_sha || "—"}
          </p>
          {branches.query.error && (
            <p role="alert" className="text-sm text-destructive">
              {extractErrorMessage(branches.query.error)}
            </p>
          )}
          {error && (
            <p role="alert" className="text-sm text-destructive">
              {error}
            </p>
          )}
          <Button
            type="submit"
            disabled={mutation.isPending || !branchRows.find((item) => item.name === branch)?.last_commit_sha}
          >
            {t(mutation.isPending ? "Creating..." : "Run pipeline")}
          </Button>
        </form>
      </FormDialog>
      <FormDialog
        open={cancelOpen}
        onClose={() => {
          if (!mutation.isPending) setCancelOpen(false);
        }}
        title={t("Cancel pipeline")}
      >
        <p className="mb-4 text-sm">{t("Cancel all pending and running jobs in this pipeline?")}</p>
        <Button variant="destructive" disabled={mutation.isPending} onClick={() => void action("cancel")}>
          {t("Cancel pipeline")}
        </Button>
      </FormDialog>
    </div>
  );
};
