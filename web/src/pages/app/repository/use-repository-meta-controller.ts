import { useMemo, useState } from "react";
import { useCustom, useCustomMutation, useDelete, useList } from "@refinedev/core";
import type {
  OrganizationView,
  RepositoryBranchProtectionPatch,
  RepositoryBranchProtectionRuleType,
  RepositoryBranchView,
  RepositoryCommitView,
  RepositoryView,
} from "@/pages/types";
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

  const { mutate: deleteRepository, mutation: { isPending: isDeleting } } = useDelete<RepositoryView>();
  const { mutateAsync: createBranch, mutation: { isPending: isCreatingBranch } } = useCustomMutation();
  const { mutateAsync: patchBranchProtection, mutation: { isPending: isUpdatingBranch } } = useCustomMutation();
  const { mutateAsync: deleteBranch, mutation: { isPending: isDeletingBranch } } = useCustomMutation();

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
    () => (orgQuery.result.data ?? []).find((item) => item.id === organizationId),
    [orgQuery.result.data, organizationId],
  );
  const repository = useMemo(
    () => (repoQuery.result.data ?? []).find((item) => item.id === repoId) ?? null,
    [repoQuery.result.data, repoId],
  );
  const branches = branchesQuery.result.data ?? [];
  const commits = commitsQuery.result.data ?? [];

  const loadBranches = async (): Promise<void> => {
    const result = await branchesQuery.query.refetch();
    if (result.error) {
      setActionError(extractErrorMessage(result.error));
      return;
    }
    setActionError(null);
  };

  const loadCommits = async (): Promise<void> => {
    const result = await commitsQuery.query.refetch();
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

  const updateBranchProtection = async (branch: RepositoryBranchView, patch: RepositoryBranchProtectionPatch) => {
    setActionError(null);
    try {
      const current = branch.protection;
      await patchBranchProtection({
        url: `/projects/${repoId}/repository/branch-protections/${encodeURIComponent(branch.name)}`,
        method: "patch",
        values: {
          rule_type: patch.rule_type ?? current?.rule_type ?? defaultProtectionRuleType(branch.name),
          push_access_level: patch.push_access_level ?? current?.push_access_level ?? "no_one",
          merge_access_level: patch.merge_access_level ?? current?.merge_access_level ?? "maintainer",
          require_merge_request: patch.require_merge_request ?? current?.require_merge_request ?? true,
          require_pipeline_success: patch.require_pipeline_success ?? current?.require_pipeline_success ?? false,
          allow_force_push: patch.allow_force_push ?? current?.allow_force_push ?? false,
          allow_delete: patch.allow_delete ?? current?.allow_delete ?? false,
        },
      });
      await loadBranches();
    } catch (error) {
      setActionError(extractErrorMessage(error));
    }
  };

  const removeBranch = async (branch: RepositoryBranchView) => {
    setActionError(null);
    try {
      await deleteBranch({
        url: `/projects/${repoId}/repository/branches/${encodeURIComponent(branch.name)}`,
        method: "delete",
        values: {},
      });
      if (branchFilter === branch.name) {
        setBranchFilter("all");
      }
      await Promise.all([loadBranches(), loadCommits()]);
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

  const submitDelete = (confirmation: string) => {
    if (!repository) {
      return;
    }
    setActionError(null);
    deleteRepository(
      { resource: "my-projects", id: repository.id, meta: { confirmation } },
      { onSuccess: onDeleted, onError: (error) => setActionError(extractErrorMessage(error)) },
    );
  };

  const isLoading = orgQuery.query.isLoading || repoQuery.query.isLoading;
  const queryError = orgQuery.query.error ?? repoQuery.query.error ?? branchesQuery.query.error ?? commitsQuery.query.error;
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
    isLoadingBranches: branchesQuery.query.isFetching,
    isLoadingCommits: commitsQuery.query.isFetching,
    isUpdatingBranch,
    isCreatingBranch,
    isDeletingBranch,
    isDeleting,
    setBranchFilter,
    setNewBranchName,
    loadBranches,
    loadCommits,
    submitCreateBranch,
    toggleBranchProtection,
    updateBranchProtection,
    removeBranch,
    copyCloneUrl,
    submitDelete,
  };
};

const defaultProtectionRuleType = (branchName: string): RepositoryBranchProtectionRuleType =>
  branchName.includes("*") || branchName.includes("?") ? "pattern" : "exact";
