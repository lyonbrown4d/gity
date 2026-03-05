import { useEffect, useMemo, useState } from "react";
import { useDelete, useList } from "@refinedev/core";
import { apiRequest } from "@/lib/api";
import type { OrganizationView, RepositoryBranchView, RepositoryCommitView, RepositoryView } from "@/pages/types";
import { extractErrorMessage } from "./repository-utils";

interface UseRepositoryMetaControllerArgs {
  organizationId: string;
  repoId: string;
  t: (text: string) => string;
  onDeleted: () => void;
}

export function useRepositoryMetaController({
  organizationId,
  repoId,
  t,
  onDeleted,
}: UseRepositoryMetaControllerArgs) {
  const [branchFilter, setBranchFilter] = useState("all");
  const [newBranchName, setNewBranchName] = useState("");
  const [branches, setBranches] = useState<RepositoryBranchView[]>([]);
  const [commits, setCommits] = useState<RepositoryCommitView[]>([]);
  const [isLoadingBranches, setLoadingBranches] = useState(false);
  const [isLoadingCommits, setLoadingCommits] = useState(false);
  const [isUpdatingBranch, setUpdatingBranch] = useState(false);
  const [isCreatingBranch, setCreatingBranch] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const { mutate: deleteRepository, isLoading: isDeleting } = useDelete<RepositoryView>();
  const orgQuery = useList<OrganizationView>({ resource: "organizations" });
  const repoQuery = useList<RepositoryView>({
    resource: "my-repositories",
    meta: { organization_id: organizationId },
    queryOptions: { enabled: Boolean(organizationId) },
  });

  const organization = useMemo(
    () => (orgQuery.data?.data ?? []).find((item) => item.id === organizationId),
    [orgQuery.data?.data, organizationId],
  );
  const repository = useMemo(
    () => (repoQuery.data?.data ?? []).find((item) => item.id === repoId) ?? null,
    [repoQuery.data?.data, repoId],
  );

  const loadBranches = async () => {
    setLoadingBranches(true);
    try {
      setBranches(await apiRequest<RepositoryBranchView[]>(`/repos/${repoId}/branches`));
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingBranches(false);
    }
  };

  const loadCommits = async () => {
    setLoadingCommits(true);
    try {
      const query = new URLSearchParams({ limit: "50" });
      if (branchFilter !== "all") {
        query.set("branch_name", branchFilter);
      }
      setCommits(await apiRequest<RepositoryCommitView[]>(`/repos/${repoId}/commits?${query.toString()}`));
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingCommits(false);
    }
  };

  const submitCreateBranch = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!newBranchName.trim()) {
      return;
    }
    setActionError(null);
    setCreatingBranch(true);
    try {
      await apiRequest(`/repos/${repoId}/branches`, {
        method: "POST",
        body: JSON.stringify({ name: newBranchName.trim() }),
      });
      setNewBranchName("");
      await Promise.all([loadBranches(), loadCommits()]);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setCreatingBranch(false);
    }
  };

  const toggleBranchProtection = async (branch: RepositoryBranchView, protect: boolean) => {
    setActionError(null);
    setUpdatingBranch(true);
    try {
      const op = protect ? "protect" : "unprotect";
      await apiRequest(`/repos/${repoId}/branches/${encodeURIComponent(branch.name)}/${op}`, { method: "POST" });
      await loadBranches();
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setUpdatingBranch(false);
    }
  };

  const copyCloneUrl = async () => {
    if (!repository) {
      return;
    }
    try {
      await navigator.clipboard.writeText(repository.clone_http_url);
    } catch {
      setActionError(t("Failed to copy clone URL"));
    }
  };

  const submitDelete = () => {
    if (!repository) {
      return;
    }
    const confirmText = t("Delete repository \"{name}\"?").replace("{name}", repository.name);
    if (!window.confirm(confirmText)) {
      return;
    }
    setActionError(null);
    deleteRepository(
      { resource: "my-repositories", id: repository.id },
      { onSuccess: onDeleted, onError: (error) => setActionError(extractErrorMessage(error)) },
    );
  };

  useEffect(() => {
    if (repoId) {
      void loadBranches();
    }
  }, [repoId]);

  useEffect(() => {
    if (repoId) {
      void loadCommits();
    }
  }, [repoId, branchFilter]);

  const isLoading = orgQuery.isLoading || repoQuery.isLoading;
  const errorMessage = actionError
    ?? (orgQuery.error instanceof Error
      ? orgQuery.error.message
      : repoQuery.error instanceof Error
        ? repoQuery.error.message
        : null);

  return {
    organization,
    repository,
    isLoading,
    errorMessage,
    setActionError,
    branches,
    commits,
    branchFilter,
    newBranchName,
    isLoadingBranches,
    isLoadingCommits,
    isUpdatingBranch,
    isCreatingBranch,
    isDeleting,
    setBranchFilter,
    setNewBranchName,
    loadBranches,
    loadCommits,
    submitCreateBranch,
    toggleBranchProtection,
    copyCloneUrl,
    submitDelete,
  };
}
