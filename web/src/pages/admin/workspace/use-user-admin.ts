import { useMemo, useState } from "react";
import { useCreate, useDelete, useTable, useUpdate } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import type { UserView } from "@/pages/types";

type UserRole = "user" | "super-admin";
type UserStatus = "active" | "disabled";

export function useUserAdmin() {
  const { t } = useI18n();
  const [actionError, setActionError] = useState<string | null>(null);
  const [createForm, setCreateForm] = useState({
    open: false, username: "", email: "", password: "", role: "user" as UserRole,
  });
  const [editForm, setEditForm] = useState({
    open: false,
    user: null as UserView | null,
    username: "",
    email: "",
    password: "",
    status: "active" as UserStatus,
    role: "user" as UserRole,
  });
  const table = useTable<UserView>({
    resource: "users",
    pagination: { mode: "server", currentPage: 1, pageSize: 20 },
    sorters: { mode: "off" },
    filters: { mode: "off" },
  });
  const createMutation = useCreate<UserView>();
  const updateMutation = useUpdate<UserView>();
  const deleteMutation = useDelete<UserView>();
  const users = table.tableQuery.data?.data ?? [];
  const visibleStats = useMemo(() => ({
    superAdminCount: users.filter((user) => user.is_super_admin).length,
    activeCount: users.filter((user) => user.status.toLowerCase() === "active").length,
  }), [users]);
  const queryError = table.tableQuery.error;
  const errorMessage = actionError ?? (queryError instanceof Error ? queryError.message : null);
  const closeCreate = () => setCreateForm({ open: false, username: "", email: "", password: "", role: "user" });
  const closeEdit = () => setEditForm({
    open: false, user: null, username: "", email: "", password: "", status: "active", role: "user",
  });
  const fail = (error: unknown, fallback: string) => {
    setActionError(error instanceof Error ? error.message : t(fallback));
  };

  const openEdit = (user: UserView) => setEditForm({
    open: true,
    user,
    username: user.username,
    email: user.email,
    password: "",
    status: user.status.toLowerCase() === "disabled" ? "disabled" : "active",
    role: user.is_super_admin ? "super-admin" : "user",
  });

  const submitCreate = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);
    createMutation.mutate(
      {
        resource: "users",
        values: {
          username: createForm.username,
          email: createForm.email,
          password: createForm.password,
          is_super_admin: createForm.role === "super-admin",
        },
      },
      {
        onSuccess: async () => {
          closeCreate();
          await table.tableQuery.refetch();
        },
        onError: (error) => fail(error, "Failed to create user"),
      },
    );
  };

  const submitUpdate = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!editForm.user) return;
    setActionError(null);
    updateMutation.mutate(
      {
        resource: "users",
        id: editForm.user.id,
        values: {
          username: editForm.username,
          email: editForm.email,
          password: editForm.password.trim() ? editForm.password : undefined,
          status: editForm.status,
          is_super_admin: editForm.role === "super-admin",
        },
      },
      {
        onSuccess: async () => {
          closeEdit();
          await table.tableQuery.refetch();
        },
        onError: (error) => fail(error, "Failed to update user"),
      },
    );
  };

  const submitDelete = (user: UserView) => {
    setActionError(null);
    deleteMutation.mutate(
      { resource: "users", id: user.id },
      {
        onSuccess: async () => table.tableQuery.refetch(),
        onError: (error) => fail(error, "Failed to delete user"),
      },
    );
  };

  const previousPage = () => {
    if (table.currentPage > 1) table.setCurrentPage((page) => Math.max(1, page - 1));
  };
  const nextPage = () => {
    if (table.currentPage < table.pageCount) {
      table.setCurrentPage((page) => Math.min(table.pageCount, page + 1));
    }
  };

  return {
    users,
    totalUsers: table.tableQuery.data?.total ?? 0,
    visibleStats,
    isLoading: table.tableQuery.isLoading,
    errorMessage,
    currentPage: table.currentPage,
    pageSize: table.pageSize,
    pageCount: table.pageCount,
    setPageSize: table.setPageSize,
    previousPage,
    nextPage,
    createForm,
    setCreateForm,
    editForm,
    setEditForm,
    openEdit,
    closeCreate,
    closeEdit,
    submitCreate,
    submitUpdate,
    submitDelete,
    isCreating: createMutation.mutation.isPending,
    isUpdating: updateMutation.mutation.isPending,
    isDeleting: deleteMutation.mutation.isPending,
  };
}

export type UserAdminController = ReturnType<typeof useUserAdmin>;
