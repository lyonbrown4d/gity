import { useState } from "react";
import { useCreate, useList } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Modal } from "@/components/ui/modal";
import { Skeleton } from "@/components/ui/skeleton";
import type { UserView } from "@/pages/types";

export function AdminUsersPage(): JSX.Element {
  const { t } = useI18n();
  const [isCreateModalOpen, setCreateModalOpen] = useState(false);
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);

  const { data, isLoading, error, refetch } = useList<UserView>({
    resource: "users",
    pagination: {
      pageSize: 100,
    },
  });
  const { mutate: createUser, isLoading: isCreating } = useCreate<UserView>();

  const users = data?.data ?? [];
  const superAdminCount = users.filter((user) => user.is_super_admin).length;
  const activeCount = users.filter((user) => user.status.toLowerCase() === "active").length;
  const errorMessage = actionError ?? (error instanceof Error ? error.message : null);

  const resetCreateForm = () => {
    setUsername("");
    setEmail("");
    setPassword("");
    setCreateModalOpen(false);
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
          await refetch();
        },
        onError: (createError) => {
          setActionError(createError instanceof Error ? createError.message : t("Failed to create user"));
        },
      },
    );
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
          <Badge variant="secondary">{users.length} {t("total")}</Badge>
          <Badge variant="outline">{activeCount} {t("active")}</Badge>
          <Badge>
            {superAdminCount} {t("super-admin")}
          </Badge>
        </CardContent>
      </Card>

      <Card className="card-enter">
        <CardHeader>
          <CardTitle>{t("User Directory")}</CardTitle>
          <CardDescription>{t("Identity, status, and privilege level.")}</CardDescription>
        </CardHeader>
        <CardContent>
          {errorMessage ? (
            <p className="mb-3 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
              {errorMessage}
            </p>
          ) : null}

          {isLoading ? (
            <div className="space-y-2">
              <Skeleton className="h-12 w-full" />
              <Skeleton className="h-12 w-full" />
              <Skeleton className="h-12 w-full" />
            </div>
          ) : null}

          <div className="space-y-3">
            {users.map((user) => (
              <div key={user.id} className="rounded-lg border p-3">
                <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                  <div className="min-w-0">
                    <p className="truncate font-medium">{user.username}</p>
                    <p className="truncate text-xs text-muted-foreground">
                      {user.email} · {user.id}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant="secondary">{user.status}</Badge>
                    {user.is_super_admin ? <Badge>{t("super-admin")}</Badge> : null}
                  </div>
                </div>
              </div>
            ))}
            {users.length === 0 && !errorMessage && !isLoading ? (
              <p className="text-sm text-muted-foreground">{t("No users found.")}</p>
            ) : null}
          </div>
        </CardContent>
      </Card>

      <Modal open={isCreateModalOpen} onClose={resetCreateForm} title={t("Create User")}>
        <form className="grid gap-3" onSubmit={submitCreateUser}>
          <div className="space-y-2">
            <Label htmlFor="create-user-username">{t("Username")}</Label>
            <Input
              id="create-user-username"
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              placeholder="new-user"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-user-email">{t("Email")}</Label>
            <Input
              id="create-user-email"
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="new-user@example.com"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-user-password">{t("Password")}</Label>
            <Input
              id="create-user-password"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="StrongPassword123"
              required
            />
          </div>
          <div className="flex items-center justify-end gap-2">
            <Button type="button" variant="outline" onClick={resetCreateForm}>
              {t("Cancel")}
            </Button>
            <Button type="submit" disabled={isCreating}>
              {isCreating ? t("Creating...") : t("Create")}
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
