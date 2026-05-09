import { useMemo, useState } from "react";
import { useCreate, useDelete, useTable, useUpdate } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { FormDialog as Modal } from "@/components/common/form-dialog";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { UserView } from "@/pages/types";

export function AdminUsersPage(): JSX.Element {
  const { t } = useI18n();
  const [isCreateModalOpen, setCreateModalOpen] = useState(false);
  const [isEditModalOpen, setEditModalOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<UserView | null>(null);
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [editUsername, setEditUsername] = useState("");
  const [editEmail, setEditEmail] = useState("");
  const [editPassword, setEditPassword] = useState("");
  const [editStatus, setEditStatus] = useState<"active" | "disabled">("active");
  const [actionError, setActionError] = useState<string | null>(null);

  const { tableQuery, current, setCurrent, pageSize, setPageSize, pageCount } = useTable<UserView>({
    resource: "users",
    pagination: {
      mode: "server",
      current: 1,
      pageSize: 20,
    },
    sorters: { mode: "off" },
    filters: { mode: "off" },
  });

  const users = tableQuery.data?.data ?? [];
  const totalUsers = tableQuery.data?.total ?? 0;
  const isLoading = tableQuery.isLoading;
  const error = tableQuery.error;

  const { mutate: createUser, isLoading: isCreating } = useCreate<UserView>();
  const { mutate: updateUser, isLoading: isUpdating } = useUpdate<UserView>();
  const { mutate: deleteUser, isLoading: isDeleting } = useDelete<UserView>();

  const visibleStats = useMemo(() => {
    const superAdminCount = users.filter((user) => user.is_super_admin).length;
    const activeCount = users.filter((user) => user.status.toLowerCase() === "active").length;
    return { superAdminCount, activeCount };
  }, [users]);

  const errorMessage = actionError ?? (error instanceof Error ? error.message : null);

  const resetCreateForm = () => {
    setUsername("");
    setEmail("");
    setPassword("");
    setCreateModalOpen(false);
  };

  const resetEditForm = () => {
    setEditingUser(null);
    setEditUsername("");
    setEditEmail("");
    setEditPassword("");
    setEditStatus("active");
    setEditModalOpen(false);
  };

  const openEditModal = (user: UserView) => {
    setEditingUser(user);
    setEditUsername(user.username);
    setEditEmail(user.email);
    setEditPassword("");
    setEditStatus(user.status.toLowerCase() === "disabled" ? "disabled" : "active");
    setEditModalOpen(true);
  };

  const submitCreateUser = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);

    createUser(
      {
        resource: "users",
        values: {
          username,
          email,
          password,
        },
      },
      {
        onSuccess: async () => {
          resetCreateForm();
          await tableQuery.refetch();
        },
        onError: (createError) => {
          setActionError(createError instanceof Error ? createError.message : t("Failed to create user"));
        },
      },
    );
  };

  const submitUpdateUser = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!editingUser) {
      return;
    }

    setActionError(null);
    updateUser(
      {
        resource: "users",
        id: editingUser.id,
        values: {
          username: editUsername,
          email: editEmail,
          password: editPassword.trim() ? editPassword : undefined,
          status: editStatus,
        },
      },
      {
        onSuccess: async () => {
          resetEditForm();
          await tableQuery.refetch();
        },
        onError: (updateError) => {
          setActionError(updateError instanceof Error ? updateError.message : t("Failed to update user"));
        },
      },
    );
  };

  const submitDeleteUser = (user: UserView) => {
    setActionError(null);
    deleteUser(
      {
        resource: "users",
        id: user.id,
      },
      {
        onSuccess: async () => {
          await tableQuery.refetch();
        },
        onError: (deleteError) => {
          setActionError(deleteError instanceof Error ? deleteError.message : t("Failed to delete user"));
        },
      },
    );
  };

  const handlePrevPage = () => {
    if (current > 1) {
      setCurrent((previous) => Math.max(1, previous - 1));
    }
  };

  const handleNextPage = () => {
    if (current < pageCount) {
      setCurrent((previous) => Math.min(pageCount, previous + 1));
    }
  };

  return (
    <div className="space-y-4 page-enter">
      <Card className="card-enter">
        <CardHeader>
          <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <CardTitle>{t("Users")}</CardTitle>
              <CardDescription>{t("User accounts in current admin scope.")}</CardDescription>
            </div>
            <Button type="button" onClick={() => setCreateModalOpen(true)} className="action-pop">
              {t("New User")}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-2">
          <Badge variant="secondary">{totalUsers} {t("total")}</Badge>
          <Badge variant="outline">{visibleStats.activeCount} {t("active")} (page)</Badge>
          <Badge>{visibleStats.superAdminCount} {t("super-admin")} (page)</Badge>
        </CardContent>
      </Card>

      <Card className="card-enter">
        <CardHeader>
          <CardTitle>{t("User Directory")}</CardTitle>
          <CardDescription>{t("Identity, status, and privilege level.")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {errorMessage ? (
            <Alert variant="destructive">
              <AlertDescription>{errorMessage}</AlertDescription>
            </Alert>
          ) : null}

          {isLoading ? (
            <div className="space-y-2">
              <Skeleton className="h-12 w-full" />
              <Skeleton className="h-12 w-full" />
              <Skeleton className="h-12 w-full" />
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("Username")}</TableHead>
                  <TableHead>{t("Email")}</TableHead>
                  <TableHead>{t("Status")}</TableHead>
                  <TableHead>{t("Role")}</TableHead>
                  <TableHead className="text-right">{t("Actions")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map((user) => (
                  <TableRow key={user.id}>
                    <TableCell className="font-medium">{user.username}</TableCell>
                    <TableCell>
                      <div className="space-y-1">
                        <p>{user.email}</p>
                        <p className="text-xs text-muted-foreground">{user.id}</p>
                      </div>
                    </TableCell>
                    <TableCell><Badge variant="secondary">{user.status}</Badge></TableCell>
                    <TableCell>
                      {user.is_super_admin ? <Badge>{t("super-admin")}</Badge> : <Badge variant="outline">user</Badge>}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center justify-end gap-2">
                        <Button type="button" size="sm" variant="outline" onClick={() => openEditModal(user)}>
                          {t("Edit")}
                        </Button>
                        <ConfirmAction
                          title={t("Delete user \"{name}\"?").replace("{name}", user.username)}
                          description={t("This action cannot be undone.")}
                          confirmLabel={t("Delete")}
                          cancelLabel={t("Cancel")}
                          onConfirm={() => submitDeleteUser(user)}
                        >
                          <Button
                            type="button"
                            size="sm"
                            variant="destructive"
                            disabled={isDeleting || user.is_super_admin}
                          >
                            {t("Delete")}
                          </Button>
                        </ConfirmAction>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
                {users.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-sm text-muted-foreground">
                      {t("No users found.")}
                    </TableCell>
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          )}

          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="text-sm text-muted-foreground">
              {t("Page")} {current} / {Math.max(pageCount, 1)}
            </div>
            <div className="flex items-center gap-2">
              <Label htmlFor="users-page-size" className="text-xs">{t("Page size")}</Label>
              <Select value={String(pageSize)} onValueChange={(value) => setPageSize(Number(value))}>
                <SelectTrigger id="users-page-size" className="w-24">
                  <SelectValue placeholder={t("Page size")} />
                </SelectTrigger>
                <SelectContent>
                  {[10, 20, 50, 100].map((value) => (
                    <SelectItem key={value} value={String(value)}>{value}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button type="button" variant="outline" size="sm" onClick={handlePrevPage} disabled={current <= 1}>
                {t("Prev")}
              </Button>
              <Button type="button" variant="outline" size="sm" onClick={handleNextPage} disabled={current >= pageCount}>
                {t("Next")}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <Modal open={isCreateModalOpen} onClose={resetCreateForm} title={t("Create User")}>
        <form className="grid gap-3" onSubmit={submitCreateUser}>
          <div className="space-y-2">
            <Label htmlFor="create-user-username">{t("Username")}</Label>
            <Input id="create-user-username" value={username} onChange={(event) => setUsername(event.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-user-email">{t("Email")}</Label>
            <Input id="create-user-email" type="email" value={email} onChange={(event) => setEmail(event.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-user-password">{t("Password")}</Label>
            <Input id="create-user-password" type="password" value={password} onChange={(event) => setPassword(event.target.value)} required />
          </div>
          <div className="flex items-center justify-end gap-2">
            <Button type="button" variant="outline" onClick={resetCreateForm}>{t("Cancel")}</Button>
            <Button type="submit" disabled={isCreating}>{isCreating ? t("Creating...") : t("Create")}</Button>
          </div>
        </form>
      </Modal>

      <Modal open={isEditModalOpen} onClose={resetEditForm} title={t("Edit User")}>
        <form className="grid gap-3" onSubmit={submitUpdateUser}>
          <div className="space-y-2">
            <Label htmlFor="edit-user-username">{t("Username")}</Label>
            <Input id="edit-user-username" value={editUsername} onChange={(event) => setEditUsername(event.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-user-email">{t("Email")}</Label>
            <Input id="edit-user-email" type="email" value={editEmail} onChange={(event) => setEditEmail(event.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-user-password">{t("Password (optional)")}</Label>
            <Input id="edit-user-password" type="password" value={editPassword} onChange={(event) => setEditPassword(event.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-user-status">{t("Status")}</Label>
            <Select
              value={editStatus}
              onValueChange={(value) => setEditStatus(value === "disabled" ? "disabled" : "active")}
            >
              <SelectTrigger id="edit-user-status">
                <SelectValue placeholder={t("Status")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="active">{t("active")}</SelectItem>
                <SelectItem value="disabled">{t("disabled")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-center justify-end gap-2">
            <Button type="button" variant="outline" onClick={resetEditForm}>{t("Cancel")}</Button>
            <Button type="submit" disabled={isUpdating}>{isUpdating ? t("Saving...") : t("Save")}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
