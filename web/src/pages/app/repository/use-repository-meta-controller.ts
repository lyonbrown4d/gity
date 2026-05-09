import { useMemo, useState } from "react";
import { useCustom, useCustomMutation, useDelete, useList } from "@refinedev/core";
import type { OrganizationView, RepositoryBranchView, RepositoryCommitView, RepositoryView } from "@/pages/types";
import { extractErrorMessage } from "./repository-utils";

interface UseRepositoryMetaControllerArgs {
  organizationId: string;
  repoId: string;
  t: (text: string) => string;
  onDeleted: () => void;
}

export const useRepositoryMetaController = ({
  organizationId,
  repoId,
  t,
  onDeleted,
}: UseRepositoryMetaControllerArgs) => {
  const [branchFilter, setBranchFilter] = useState("all");
  const [newBranchName, setNewBranchName] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);

  const { mutate: deleteRepository, isLoading: isDeleting } = useDelete<RepositoryView>();
  const { mutateAsync: createBranch, isLoading: isCreatingBranch } = useCustomMutation();
  const { mutateAsync: patchBranchProtection, isLoading: isUpdatingBranch } = useCustomMutation();

  const orgQuery = useList<OrganizationView>({ resource: "organizations" });
  const repoQuery = useList<RepositoryView>({
    resource: "my-projects",
    meta: { organization_id: organizationId },
    queryOptions: { enabled: Boolean(organizationId) },
  });
  const commitQuery = useMemo(() => {
    if (branchFilter === "all") {
      return { limit: 50 };
    }
    return { limit: 50, branch_name: branchFilter };
  }, [branchFilter]);
  const branchesQuery = useCustom<RepositoryBranchView[]>({
    url: `/projects/${repoId}/repository/branches`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const commitsQuery = useCustom<RepositoryCommitView[]>({
    url: `/projects/${repoId}/repository/commits`,
    method: "get",
    config: { query: commitQuery },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });

  const organization = useMemo(
    () => (orgQuery.data?.data ?? []).find((item) => item.id === organizationId),
    [orgQuery.data?.data, organizationId],
  );
  const repository = useMemo(
    () => (repoQuery.data?.data ?? []).find((item) => item.id === repoId) ?? null,
    [repoQuery.data?.data, repoId],
  );
  const branches = branchesQuery.data?.data ?? [];
  const commits = commitsQuery.data?.data ?? [];

  const loadBranches = async (): Promise<void> => {
    const result = await branchesQuery.refetch();
    if (result.error) {
      setActionError(extractErrorMessage(result.error));
      return;
    }
    setActionError(null);
  };

  const loadCommits = async (): Promise<void> => {
    const result = await commitsQuery.refetch();
    if (result.error) {
      setActionError(extractErrorMessage(result.error));
      return;
    }
    setActionError(null);
  };

  const submitCreateBranch = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!newBranchName.trim()) {
      return;
    }
    setActionError(null);
    try {
      await createBranch({
        url: `/projects/${repoId}/repository/branches`,
        method: "post",
        values: { name: newBranchName.trim() },
      });
      setNewBranchName("");
      await Promise.all([loadBranches(), loadCommits()]);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    }
  };

  const toggleBranchProtection = async (branch: RepositoryBranchView, protect: boolean) => {
    setActionError(null);
    try {
      const op = protect ? "protect" : "unprotect";
      await patchBranchProtection({
        url: `/projects/${repoId}/repository/branches/${encodeURIComponent(branch.name)}/${op}`,
        method: "post",
        values: {},
      });
      await loadBranches();
    } catch (error) {
      setActionError(extractErrorMessage(error));
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
    setActionError(null);
    deleteRepository(
      { resource: "my-projects", id: repository.id },
      { onSuccess: onDeleted, onError: (error) => setActionError(extractErrorMessage(error)) },
    );
  };

  const isLoading = orgQuery.isLoading || repoQuery.isLoading;
  const queryError = orgQuery.error ?? repoQuery.error ?? branchesQuery.error ?? commitsQuery.error;
  const errorMessage = actionError ?? (queryError ? extractErrorMessage(queryError) : null);

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
    isLoadingBranches: branchesQuery.isFetching,
    isLoadingCommits: commitsQuery.isFetching,
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
};
