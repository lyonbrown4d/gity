import { lazy, Suspense, useMemo, useRef, useState } from "react";
import { CheckCircle2, Plus } from "lucide-react";
import type { OnMount } from "@monaco-editor/react";
import type { Connection } from "@xyflow/react";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/lib/theme";
import { registerEditorTheme } from "@/lib/monaco-theme";
import { connectJobs, editJob, parseConfig, stagesOf, type CIJob } from "./config-model";
import { PipelineGraph } from "./pipeline-graph";
import { JobInspector } from "./job-inspector";
import type { RepositoryPermissions } from "../repository-permissions";
import { branchWriteBlock } from "./branch-access";
import { useCILeaveGuard } from "./use-ci-leave-guard";
import { EditorLeaveDialog } from "./editor-leave-dialog";
import { useCIFile } from "./use-ci-file";
import { CIEditorToolbar } from "./ci-editor-toolbar";
import { RecoverDraftDialog } from "./recover-draft-dialog";
import { CommitConfigDialog } from "./commit-config-dialog";
const Editor = lazy(() => import("@/components/code-editor"));
interface Props {
  repoId: string;
  defaultBranch: string;
  permissions: RepositoryPermissions;
  t: (key: string) => string;
  onClose: () => void;
}
export const CIEditor = ({ repoId, defaultBranch, permissions, t, onClose }: Props) => {
  const canWrite = permissions.repositoryPush;
  const canLint = permissions.ciWrite;
  const file = useCIFile(repoId, defaultBranch);
  const { editorTheme } = useTheme();
  const writeBlock = branchWriteBlock(permissions, file.branchInfo?.protection);
  const yamlEditor = useRef<Parameters<OnMount>[0] | null>(null);
  const errorLine = useRef(1);
  const focusError = (issue: string) => {
    const line = issue.match(/line (\d+)/);
    const job = issue.match(/^jobs\.([^.:]+)/);
    const jobLine = job
      ? file.source.split("\n").findIndex((text) => text.trimStart().startsWith(`${job[1]}:`)) + 1
      : 1;
    errorLine.current = line ? Number(line[1]) : Math.max(1, jobLine);
    setMode("yaml");
    if (mode !== "yaml") return;
    yamlEditor.current?.revealLineInCenter(errorLine.current);
    yamlEditor.current?.setPosition({ lineNumber: errorLine.current, column: 1 });
  };
  const parsed = useMemo(() => parseConfig(file.source), [file.source]);
  const [mode, setMode] = useState<"canvas" | "yaml">("canvas");
  const [selected, setSelected] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [commitOpen, setCommitOpen] = useState(false);
  const guard = useCILeaveGuard(file.dirty, file.discardDraft);
  const selectedName = selected && parsed.config?.jobs[selected] ? selected : Object.keys(parsed.config?.jobs ?? {})[0];
  const leave = guard.leave;
  const apply = (nextName: string, job: CIJob) => {
    const next = editJob(file.source, selectedName, job, nextName);
    file.setSource(next);
    setSelected(nextName);
  };
  const connect = (connection: Connection) => {
    try {
      file.setSource(connectJobs(file.source, connection.source, connection.target));
      setError(null);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure));
    }
  };
  const validConnection = (connection: Connection) => {
    try {
      connectJobs(file.source, connection.source, connection.target);
      return true;
    } catch {
      return false;
    }
  };
  const addTask = () => {
    try {
      if (!parsed.config) return;
      let name = "task";
      let count = 1;
      while (parsed.config.jobs[name]) {
        name = `task-${count}`;
        count += 1;
      }
      file.setSource(editJob(file.source, name, { stage: stagesOf(parsed.config)[0], script: ["echo 'New task'"] }));
      setSelected(name);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure));
    }
  };
  return (
    <section className="space-y-4">
      <CIEditorToolbar
        branch={file.branch}
        branches={file.branches.map((branch) => branch.name)}
        dirty={file.dirty}
        mode={mode}
        onMode={setMode}
        onBranch={(branch) => leave(() => file.changeBranch(branch))}
        onBack={onClose}
        canValidate={canLint && !file.pending && file.loaded}
        canCommit={!writeBlock && canLint && file.dirty && Boolean(parsed.config) && !file.pending && file.loaded}
        onValidate={() => void file.validate()}
        onCommit={() => setCommitOpen(true)}
        t={t}
      />
      <div className="flex justify-end">
        <Button
          size="sm"
          variant="ghost"
          disabled={file.pending || file.loading}
          onClick={() => leave(() => void file.reload())}
        >
          {t("Reload branch configuration")}
        </Button>
      </div>
      {writeBlock && (
        <p role="status" className="text-sm text-muted-foreground">
          {t(writeBlock)}
        </p>
      )}
      {file.branchInfo?.is_protected && (
        <p className="text-xs text-muted-foreground">
          {t("Protected branch: the server checks push rules when committing.")}
        </p>
      )}
      {(error || file.error) && (
        <p role="alert" className="rounded-md border border-destructive/30 p-3 text-sm text-destructive">
          {error || file.error}
        </p>
      )}
      {file.loading && <p role="status">{t("Loading configuration...")}</p>}
      {file.loaded && !file.exists && (
        <p role="status" className="text-sm text-muted-foreground">
          {t("No .gity-ci.yml exists on this branch. This is a new draft.")}
        </p>
      )}
      {file.loaded && (
        <div className="overflow-hidden rounded-lg border border-border bg-card">
          {mode === "yaml" ? (
            <Suspense fallback={<p className="p-6">{t("Loading editor...")}</p>}>
              <Editor
                height="650px"
                language="yaml"
                value={file.source}
                onChange={(value) => file.setSource(value ?? "")}
                beforeMount={registerEditorTheme}
                onMount={(editor) => {
                  yamlEditor.current = editor;
                  editor.revealLineInCenter(errorLine.current);
                  editor.setPosition({ lineNumber: errorLine.current, column: 1 });
                }}
                theme={editorTheme}
                options={{
                  minimap: { enabled: false },
                  fontSize: 13,
                  wordWrap: "on",
                  scrollBeyondLastLine: false,
                  readOnly: !canWrite,
                }}
              />
            </Suspense>
          ) : parsed.config ? (
            <div className="flex flex-col lg:flex-row">
              <aside className="w-full shrink-0 border-b border-border p-4 lg:w-44 lg:border-b-0 lg:border-r">
                <h3 className="mb-4 font-semibold">{t("Tasks")}</h3>
                <Button
                  className="mb-4 w-full"
                  size="sm"
                  variant="outline"
                  disabled={!canWrite || Object.keys(parsed.config.jobs).length >= 200}
                  onClick={addTask}
                >
                  <Plus className="mr-1 size-4" />
                  {t("Add task")}
                </Button>
                <nav aria-label={t("Tasks")} className="max-h-[480px] space-y-1 overflow-auto">
                  {Object.entries(parsed.config.jobs).map(([name, job]) => (
                    <button
                      key={name}
                      className={`w-full rounded-md px-2 py-3 text-left text-sm hover:bg-muted ${name === selectedName ? "bg-primary/10 text-primary" : ""}`}
                      onClick={() => setSelected(name)}
                    >
                      <span className="block truncate font-medium">{name}</span>
                      <span className="text-xs text-muted-foreground">{job.stage ?? "test"}</span>
                    </button>
                  ))}
                </nav>
              </aside>
              <PipelineGraph
                config={parsed.config}
                selected={selectedName}
                t={t}
                readonly={!canWrite}
                onSelect={setSelected}
                onConnect={connect}
                isValidConnection={validConnection}
              />
              {selectedName && canWrite && (
                <JobInspector
                  key={`${selectedName}:${file.source}`}
                  name={selectedName}
                  config={parsed.config}
                  t={t}
                  onApply={apply}
                  onRemove={() => {
                    try {
                      file.setSource(editJob(file.source, selectedName, null));
                      setSelected(null);
                    } catch (failure) {
                      setError(String(failure));
                    }
                  }}
                />
              )}
            </div>
          ) : (
            <div className="space-y-3 p-6">
              <p className="font-medium">{t("Fix YAML before opening the canvas")}</p>
              <Button variant="outline" onClick={() => setMode("yaml")}>
                {t("Edit YAML")}
              </Button>
            </div>
          )}
          <footer className="border-t border-border p-3 text-xs text-muted-foreground">
            {parsed.config
              ? `${Object.keys(parsed.config.jobs).length} ${t("Tasks")} · ${t("Canvas edits update YAML. Commit to apply changes.")}`
              : t("The original YAML is preserved while errors are present.")}
          </footer>
        </div>
      )}
      {file.loaded && parsed.errors.length > 0 && (
        <ul role="alert" className="space-y-1 rounded-md border border-destructive/30 p-4 text-xs text-destructive">
          {parsed.errors.map((issue, index) => (
            <li key={index}>
              <button className="text-left hover:underline" title={t("Open in YAML")} onClick={() => focusError(issue)}>
                {issue}
              </button>
            </li>
          ))}
        </ul>
      )}
      {file.lint && (
        <div role="status" className="text-sm">
          {file.lint.length ? (
            file.lint.join("\n")
          ) : (
            <span className="flex items-center gap-2 text-primary">
              <CheckCircle2 className="size-4" />
              {t("Configuration is valid")}
            </span>
          )}
        </div>
      )}
      <RecoverDraftDialog
        open={Boolean(file.recoverableDraft)}
        head={file.recoverableDraft?.head}
        onRestore={file.restoreDraft}
        onDiscard={() => {
          file.discardDraft();
          file.resolveDraft();
        }}
        t={t}
      />
      <CommitConfigDialog
        open={commitOpen}
        onClose={() => setCommitOpen(false)}
        branch={file.branch}
        head={file.head}
        original={file.original}
        source={file.source}
        pending={file.pending}
        error={file.error || file.lint?.join("\n") || null}
        onSave={file.save}
        t={t}
      />
      <EditorLeaveDialog open={guard.open} onKeep={guard.keep} onDiscard={guard.discard} t={t} />
    </section>
  );
};
