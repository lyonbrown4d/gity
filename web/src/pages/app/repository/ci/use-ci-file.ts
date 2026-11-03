import { useEffect, useRef, useState } from "react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import type { RepositoryBlobView, RepositoryBranchView } from "@/pages/types";
import { extractErrorMessage } from "../issues-utils";
import { useCIDraft } from "./use-ci-draft";
import { defaultConfig } from "./config-model";
import type { CiPipelineLintResult } from "@/lib/generated/model/ciPipelineLintResult";

export const useCIFile = (repoId: string, defaultBranch: string) => {
  const [branch, setBranch] = useState(defaultBranch || "main");
  const [source, setSource] = useState("");
  const [original, setOriginal] = useState("");
  const [head, setHead] = useState("");
  const [exists, setExists] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lint, setLint] = useState<{ source: string; errors: string[] } | null>(null);
  const saving = useRef(false);
  const branches = useCustom<RepositoryBranchView[]>({
    url: `/projects/${repoId}/repository/branches`,
    method: "get",
    queryOptions: { refetchOnWindowFocus: false },
  });
  const blob = useCustom<RepositoryBlobView>({
    url: `/projects/${repoId}/repository/blob`,
    method: "get",
    config: { query: { branch_name: branch, path: ".gity-ci.yml" } },
    queryOptions: { enabled: Boolean(branch), refetchOnWindowFocus: false, retry: false },
  });
  const lintMutation = useCustomMutation<CiPipelineLintResult>();
  const saveMutation = useCustomMutation<{ commit_sha: string }>();
  const pending = lintMutation.mutation.isPending || saveMutation.mutation.isPending;
  const dirty = loaded && source !== original;
  const draft = useCIDraft(`gity.ci-draft:${repoId}:${branch}`, loaded, { source, original, head, exists });
  const branchRows = Array.isArray(branches.result.data) ? branches.result.data : [];
  const branchInfo = branchRows.find((item) => item.name === branch);
  const missing = blob.query.error?.statusCode === 404;
  useEffect(() => {
    if (loaded || !branchInfo?.last_commit_sha || (!blob.query.isSuccess && !missing)) return;
    if (blob.result.data?.is_binary) return;
    const text = missing ? defaultConfig : (blob.result.data?.content ?? "");
    setSource(text);
    setOriginal(missing ? "" : text);
    setHead(branchInfo.last_commit_sha);
    setExists(!missing);
    setLoaded(true);
  }, [loaded, branchInfo, blob.query.isSuccess, blob.result.data, missing]);
  useEffect(() => {
    if (!dirty) return;
    const beforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [dirty]);
  const changeBranch = (value: string) => {
    setBranch(value);
    setLoaded(false);
    setLint(null);
    setError(null);
  };
  const validate = async (): Promise<boolean> => {
    setError(null);
    try {
      const response = await lintMutation.mutateAsync({
        url: `/projects/${repoId}/ci/lint`,
        method: "post",
        values: { config_content: source },
      });
      setLint({ source, errors: response.data.errors ?? [] });
      return response.data.valid;
    } catch (failure) {
      setError(extractErrorMessage(failure));
      return false;
    }
  };
  const save = async (message: string): Promise<boolean> => {
    if (saving.current || pending || !message.trim() || !head) return false;
    saving.current = true;
    setError(null);
    try {
      if (!(await validate())) return false;
      const response = await saveMutation.mutateAsync({
        url: `/projects/${repoId}/repository/files`,
        method: "post",
        values: {
          branch_name: branch,
          path: ".gity-ci.yml",
          content: source,
          message: message.trim(),
          update_existing: exists,
          skip_pipeline: true,
          expected_head_sha: head,
        },
      });
      setHead(response.data.commit_sha);
      setOriginal(source);
      setExists(true);
      return true;
    } catch (failure) {
      setError(extractErrorMessage(failure));
      return false;
    } finally {
      saving.current = false;
    }
  };
  const reload = async () => {
    const [branchResult, fileResult] = await Promise.all([branches.query.refetch(), blob.query.refetch()]);
    if (branchResult.error || (fileResult.error && fileResult.error.statusCode !== 404)) {
      setError(extractErrorMessage(branchResult.error ?? fileResult.error));
      return;
    }
    setLoaded(false);
    setError(null);
    setLint(null);
  };
  return {
    branch,
    source,
    original,
    head,
    exists,
    loaded,
    dirty,
    branchInfo,
    branches: branchRows,
    setSource: (value: string) => {
      setSource(value);
      setLint(null);
    },
    changeBranch,
    reload,
    validate,
    save,
    pending,
    recoverableDraft: draft.recoverable,
    discardDraft: draft.discard,
    resolveDraft: draft.resolve,
    restoreDraft: () => {
      if (!draft.recoverable) return;
      setSource(draft.recoverable.source);
      setOriginal(draft.recoverable.original);
      setHead(draft.recoverable.head);
      setExists(draft.recoverable.exists);
      draft.resolve();
    },
    lint: lint?.source === source ? lint.errors : null,
    error:
      error ??
      (branches.query.error
        ? extractErrorMessage(branches.query.error)
        : !missing && blob.query.error
          ? extractErrorMessage(blob.query.error)
          : blob.result.data?.is_binary
            ? "CI configuration must be a text file"
            : !branches.query.isLoading && !branchInfo
              ? "No matching branch is available. Initialize the repository before editing CI."
              : null),
    loading: branches.query.isLoading || blob.query.isLoading,
  };
};
