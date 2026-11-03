import { useCustom } from "@refinedev/core";
import type { Blob } from "@/lib/generated/model";
import type { RepositoryView } from "@/pages/types";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { RepositorySourceCanvas } from "./repository-source-canvas";

export interface PinnedSourceTarget {
  commit: string;
  path: string;
  line: number;
}

export const parsePinnedSourceTarget = (search: string): PinnedSourceTarget | null => {
  const params = new URLSearchParams(search);
  const commit = params.get("commit_sha") ?? "";
  const path = params.get("path") ?? "";
  const line = Number(params.get("line"));
  if (!/^[a-f0-9]{40}(?:[a-f0-9]{24})?$/.test(commit) || !path || path.startsWith("/")
    || path.split("/").some((part) => part === ".." || part === "." || part === "")
    || !Number.isSafeInteger(line) || line < 1) return null;
  return { commit, path, line };
};

interface Props {
  repository: RepositoryView;
  target: PinnedSourceTarget;
  editorTheme: string;
  t: (text: string) => string;
  onBrowse: () => void;
}

export const RepositoryPinnedSource = ({ repository, target, editorTheme, t, onBrowse }: Props): JSX.Element => {
  const source = useCustom<Blob>({
    url: `/projects/${repository.id}/repository/blob`,
    method: "get",
    config: { query: { ref_name: target.commit, path: target.path } },
    queryOptions: { retry: false, staleTime: 0, gcTime: 0 },
  });
  // Hide cached source while a new authorization check is in flight.
  const blob = source.query.isSuccess && !source.query.isFetching ? source.result.data : undefined;
  return (
    <section className="overflow-hidden rounded-lg border border-border bg-card">
      <div className="flex items-center justify-between gap-3 border-b border-border px-4 py-3">
        <span className="truncate font-mono text-xs">{target.commit.slice(0, 12)} · {target.path}:{target.line}</span>
        <Button variant="outline" size="sm" onClick={onBrowse}>{t("Code")}</Button>
      </div>
      {source.query.isFetching ? <p role="status" className="p-4 text-sm text-muted-foreground">{t("Loading...")}</p> : null}
      {source.query.isError ? <Alert variant="destructive"><AlertDescription>{t("Unable to load file.")}</AlertDescription></Alert> : null}
      {blob ? <RepositorySourceCanvas
        t={t} repository={repository} selectedBlob={blob} readmePreview={null} readmePath={null}
        isEmptyRepository={false} editorTheme={editorTheme} searchLine={target.line} searchColumn={1}
        searchMatchLength={0} searchJumpRequest={1} searchTargetPath={target.path} onOpenFile={() => undefined}
      /> : null}
    </section>
  );
};
