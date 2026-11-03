import { useEffect, useMemo, useState } from "react";
import { useCreate, useDelete, useList, useUpdate } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import type { OrganizationMemberView, OrganizationView, RepositoryView } from "@/pages/types";

export const useOrganizationAdmin = () => {
  const { t } = useI18n();
  const [selectedOrg, setSelectedOrg] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);
  const [createForm, setCreateForm] = useState({ open: false, key: "", name: "" });
  const [editForm, setEditForm] = useState({ open: false, key: "", name: "" });
  const [memberForm, setMemberForm] = useState({ open: false, userId: "", role: "member" });

  const createMutation = useCreate<OrganizationView>();
  const updateMutation = useUpdate<OrganizationView>();
  const deleteMutation = useDelete<OrganizationView>();
  const memberMutation = useCreate<OrganizationMemberView>();
  const orgQuery = useList<OrganizationView>({ resource: "organizations" });
  const orgs = orgQuery.result.data ?? [];

  useEffect(() => {
    if (!selectedOrg && orgs[0]) setSelectedOrg(orgs[0].id);
  }, [orgs, selectedOrg]);

  const selectedOrgModel = useMemo(
    () => orgs.find((org) => org.id === selectedOrg) ?? null,
    [orgs, selectedOrg],
  );
  const membersQuery = useList<OrganizationMemberView>({
    resource: "organization-members",
    meta: { organization_id: selectedOrg },
    queryOptions: { enabled: Boolean(selectedOrg) },
  });
  const reposQuery = useList<RepositoryView>({
    resource: "projects",
    meta: { organization_id: selectedOrg },
    queryOptions: { enabled: Boolean(selectedOrg) },
  });

  const queryError = orgQuery.query.error ?? membersQuery.query.error ?? reposQuery.query.error;
  const errorMessage = actionError ?? (queryError instanceof Error ? queryError.message : null);
  const closeCreate = () => setCreateForm({ open: false, key: "", name: "" });
  const closeEdit = () => setEditForm({ open: false, key: "", name: "" });
  const closeMember = () => setMemberForm({ open: false, userId: "", role: "member" });
  const fail = (error: unknown, fallback: string) => {
    setActionError(error instanceof Error ? error.message : t(fallback));
  };

  const openEdit = () => {
    if (selectedOrgModel) {
      setEditForm({ open: true, key: selectedOrgModel.key, name: selectedOrgModel.name });
    }
  };

  const submitCreate = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);
    createMutation.mutate(
      { resource: "organizations", values: { key: createForm.key, name: createForm.name } },
      {
        onSuccess: async (result) => {
          closeCreate();
          await orgQuery.query.refetch();
          if (result.data?.id) setSelectedOrg(String(result.data.id));
        },
        onError: (error) => fail(error, "Failed to create organization"),
      },
    );
  };

  const submitUpdate = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedOrgModel) {
      setActionError(t("Please select an organization first."));
      return;
    }
    setActionError(null);
    updateMutation.mutate(
      {
        resource: "organizations",
        id: selectedOrgModel.id,
        values: { key: editForm.key, name: editForm.name },
      },
      {
        onSuccess: async () => {
          closeEdit();
          await orgQuery.query.refetch();
        },
        onError: (error) => fail(error, "Failed to update organization"),
      },
    );
  };

  const submitDelete = () => {
    if (!selectedOrgModel) {
      setActionError(t("Please select an organization first."));
      return;
    }
    setActionError(null);
    deleteMutation.mutate(
      { resource: "organizations", id: selectedOrgModel.id },
      {
        onSuccess: async () => {
          const refreshed = await orgQuery.query.refetch();
          const next = refreshed.data?.data.find((org) => org.id !== selectedOrgModel.id);
          setSelectedOrg(next?.id ?? "");
          await membersQuery.query.refetch();
          await reposQuery.query.refetch();
        },
        onError: (error) => fail(error, "Failed to delete organization"),
      },
    );
  };

  const submitMember = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);
    if (!selectedOrg) {
      setActionError(t("Please select an organization first."));
      return;
    }
    memberMutation.mutate(
      {
        resource: "organization-members",
        values: { organization_id: selectedOrg, user_id: memberForm.userId, role: memberForm.role },
      },
      {
        onSuccess: async () => {
          closeMember();
          await membersQuery.query.refetch();
        },
        onError: (error) => fail(error, "Failed to add member"),
      },
    );
  };

  return {
    orgs,
    members: membersQuery.result.data ?? [],
    repos: reposQuery.result.data ?? [],
    selectedOrg,
    selectedOrgModel,
    setSelectedOrg,
    isLoading: orgQuery.query.isLoading || membersQuery.query.isLoading || reposQuery.query.isLoading,
    errorMessage,
    createForm,
    setCreateForm,
    editForm,
    setEditForm,
    memberForm,
    setMemberForm,
    openEdit,
    closeCreate,
    closeEdit,
    closeMember,
    submitCreate,
    submitUpdate,
    submitDelete,
    submitMember,
    isCreating: createMutation.mutation.isPending,
    isUpdating: updateMutation.mutation.isPending,
    isDeleting: deleteMutation.mutation.isPending,
    isAddingMember: memberMutation.mutation.isPending,
  };
};

export type OrganizationAdminController = ReturnType<typeof useOrganizationAdmin>;
