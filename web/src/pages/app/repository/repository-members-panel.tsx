import { useEffect, useMemo, useState } from "react";
import { Plus, RefreshCw, Trash2, UsersRound } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { RepositoryProjectMemberView, UserView } from "@/pages/types";
import { extractErrorMessage } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeOptionalString, normalizeString, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryMembersPanelProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

const projectRoles = ["guest", "reporter", "developer", "maintainer", "owner"];

export const RepositoryMembersPanel = ({ repoId, permissions, t, onError }: RepositoryMembersPanelProps): JSX.Element => {
  const membersQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/members`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const usersQuery = useCustom<RawRecord[]>({
    url: "/users",
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: addMember, mutation: { isPending: isAdding } } = useCustomMutation<RawRecord>();
  const { mutateAsync: updateMember, mutation: { isPending: isUpdating } } = useCustomMutation<RawRecord>();
  const { mutateAsync: deleteMember, mutation: { isPending: isDeleting } } = useCustomMutation<RawRecord>();
  const [selectedUserID, setSelectedUserID] = useState("");
  const [selectedRole, setSelectedRole] = useState("developer");
  const members = useMemo(
    () => resolveProjectMembers(membersQuery.result.data).map(normalizeProjectMember),
    [membersQuery.result.data],
  );
  const users = useMemo(
    () => resolveUsers(usersQuery.result.data).map(normalizeUser),
    [usersQuery.result.data],
  );
  const memberUserIDs = useMemo(() => new Set(members.map((member) => member.user_id)), [members]);
  const availableUsers = users.filter((user) => !memberUserIDs.has(user.id));
  const canAdminMembers = permissions.repositoryAdmin;
  const isBusy = isAdding || isUpdating || isDeleting;

  const reload = async () => {
    const [membersResult, usersResult] = await Promise.all([membersQuery.query.refetch(), usersQuery.query.refetch()]);
    const error = membersResult.error ?? usersResult.error;
    onError(error ? extractErrorMessage(error) : null);
  };

  const submitAddMember = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedUserID) {
      onError(t("Select a user before adding a project member."));
      return;
    }
    onError(null);
    try {
      await addMember({
        url: `/projects/${repoId}/members`,
        method: "post",
        values: {
          user_id: Number.parseInt(selectedUserID, 10),
          role: selectedRole,
        },
      });
      setSelectedUserID("");
      setSelectedRole("developer");
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitUpdateMember = async (member: RepositoryProjectMemberView, role: string) => {
    onError(null);
    try {
      await updateMember({
        url: `/projects/${repoId}/members/${member.user_id}`,
        method: "patch",
        values: {
          user_id: Number.parseInt(member.user_id, 10),
          role,
        },
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitDeleteMember = async (member: RepositoryProjectMemberView) => {
    onError(null);
    try {
      await deleteMember({
        url: `/projects/${repoId}/members/${member.user_id}`,
        method: "delete",
        values: {},
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    const error = membersQuery.query.error ?? usersQuery.query.error;
    if (error) {
      onError(extractErrorMessage(error));
    }
  }, [membersQuery.query.error, usersQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <UsersRound className="size-4" />
          {t("Project members")}
        </CardTitle>
        <CardDescription>{t("Project-specific roles override organization membership for this repository.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {!canAdminMembers ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect members, but cannot change project membership.")}</AlertDescription>
          </Alert>
        ) : null}

        <form className="grid gap-3 rounded-md border p-3 lg:grid-cols-[1fr_180px_auto]" onSubmit={submitAddMember}>
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground">{t("User")}</Label>
            <Select value={selectedUserID} onValueChange={setSelectedUserID} disabled={!canAdminMembers || isBusy || availableUsers.length === 0}>
              <SelectTrigger>
                <SelectValue placeholder={availableUsers.length === 0 ? t("No users available") : t("Select user")} />
              </SelectTrigger>
              <SelectContent>
                {availableUsers.map((user) => (
                  <SelectItem key={user.id} value={user.id}>
                    {formatUserLabel(user)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground">{t("Role")}</Label>
            <RoleSelect value={selectedRole} disabled={!canAdminMembers || isBusy} t={t} onChange={setSelectedRole} />
          </div>
          <div className="flex items-end">
            <Button type="submit" disabled={!canAdminMembers || isBusy || !selectedUserID}>
              <Plus className="size-4" />
              {isAdding ? t("Adding...") : t("Add member")}
            </Button>
          </div>
        </form>

        <div className="flex items-center justify-between gap-2">
          <p className="text-sm font-medium">{t("Explicit project members")}</p>
          <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
            <RefreshCw className="size-4" />
            {t("Reload")}
          </Button>
        </div>

        <div className="space-y-2 rounded-md border p-2">
          {membersQuery.query.isFetching && !membersQuery.query.data ? (
            <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading project members...")}</p>
          ) : null}
          {members.length === 0 ? (
            <p className="px-2 py-2 text-sm text-muted-foreground">{t("No project-specific members yet.")}</p>
          ) : null}
          {members.map((member) => (
            <div key={member.id || member.user_id} className="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3">
              <div className="min-w-0">
                <p className="font-medium">{member.display_name || member.username || `#${member.user_id}`}</p>
                <p className="text-xs text-muted-foreground">
                  @{member.username || member.user_id} · {member.email || t("No email")} · {t("Source")}: {member.source || "project"}
                </p>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="outline">{t(member.role)}</Badge>
                <div className="w-44">
                  <RoleSelect value={normalizeRole(member.role)} disabled={!canAdminMembers || isBusy} t={t} onChange={(role) => void submitUpdateMember(member, role)} />
                </div>
                <ConfirmAction
                  title={t("Remove project member?")}
                  description={t("This removes only the project-specific role. Organization membership is not changed.")}
                  confirmLabel={t("Remove")}
                  cancelLabel={t("Cancel")}
                  onConfirm={() => void submitDeleteMember(member)}
                >
                  <Button type="button" size="sm" variant="outline" disabled={!canAdminMembers || isBusy}>
                    <Trash2 className="size-4" />
                    {isDeleting ? t("Removing...") : t("Remove")}
                  </Button>
                </ConfirmAction>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

const RoleSelect = ({
  value,
  disabled,
  t,
  onChange,
}: {
  value: string;
  disabled: boolean;
  t: (text: string) => string;
  onChange: (value: string) => void;
}) => (
  <Select value={normalizeRole(value)} onValueChange={onChange} disabled={disabled}>
    <SelectTrigger>
      <SelectValue />
    </SelectTrigger>
    <SelectContent>
      {projectRoles.map((role) => (
        <SelectItem key={role} value={role}>
          {t(role)}
        </SelectItem>
      ))}
    </SelectContent>
  </Select>
);

const normalizeRole = (value: string): string => (projectRoles.includes(value) ? value : "developer");

const resolveProjectMembers = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  return isRecord(payload) ? resolveRecordArray(payload.body ?? payload.Body) : [];
};

const resolveUsers = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  return isRecord(payload) ? resolveRecordArray(payload.body ?? payload.Body) : [];
};

const normalizeProjectMember = (raw: RawRecord): RepositoryProjectMemberView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  user_id: normalizeString(raw.user_id ?? raw.UserID),
  username: normalizeString(raw.username ?? raw.Username),
  display_name: normalizeOptionalString(raw.display_name ?? raw.DisplayName),
  email: normalizeString(raw.email ?? raw.Email),
  role: normalizeString(raw.role ?? raw.Role) || "developer",
  source: normalizeString(raw.source ?? raw.Source) || "project",
});

const normalizeUser = (raw: RawRecord): UserView => ({
  id: normalizeString(raw.id ?? raw.ID),
  username: normalizeString(raw.username ?? raw.Username),
  display_name: normalizeOptionalString(raw.display_name ?? raw.DisplayName),
  email: normalizeString(raw.email ?? raw.Email),
  status: normalizeString(raw.status ?? raw.Status),
  is_super_admin: false,
});

const formatUserLabel = (user: UserView): string => {
  const displayName = user.display_name?.trim();
  return displayName ? `${displayName} (@${user.username})` : `@${user.username}`;
};
