import { useEffect, useMemo, useState } from "react";
import { Plus, RefreshCw, Search, Trash2, UsersRound } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { RepositoryProjectMemberView, UserView } from "@/pages/types";
import { extractErrorMessage } from "./issues-utils";
import { formatUserLabel } from "./repository-user-utils";
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
  const [memberSearchQuery, setMemberSearchQuery] = useState("");
  const [roleFilter, setRoleFilter] = useState("all");
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
  const filteredMembers = useMemo(
    () => filterMembers(members, memberSearchQuery, roleFilter),
    [members, memberSearchQuery, roleFilter],
  );
  const memberStats = useMemo(
    () => ({
      total: members.length,
      maintainers: members.filter((member) => normalizeRole(member.role) === "maintainer" || normalizeRole(member.role) === "owner").length,
      developers: members.filter((member) => normalizeRole(member.role) === "developer").length,
    }),
    [members],
  );
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
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <MemberStat label={t("Direct members")} value={memberStats.total} />
          <MemberStat label={t("Maintainers/Owners")} value={memberStats.maintainers} />
          <MemberStat label={t("Developers")} value={memberStats.developers} />
        </div>

        {!canAdminMembers ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect members, but cannot change project membership.")}</AlertDescription>
          </Alert>
        ) : null}

        <form className="grid gap-3 rounded-md border p-3 lg:grid-cols-[1fr_180px_auto]" onSubmit={submitAddMember}>
          <div className="flex flex-col gap-1">
            <Label className="text-xs text-muted-foreground">{t("User")}</Label>
            <Select value={selectedUserID} onValueChange={setSelectedUserID} disabled={!canAdminMembers || isBusy || availableUsers.length === 0}>
              <SelectTrigger>
                <SelectValue placeholder={availableUsers.length === 0 ? t("No users available") : t("Select user")} />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  {availableUsers.map((user) => (
                    <SelectItem key={user.id} value={user.id}>
                      {formatUserLabel(user)}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-1">
            <Label className="text-xs text-muted-foreground">{t("Role")}</Label>
            <RoleSelect value={selectedRole} disabled={!canAdminMembers || isBusy} t={t} onChange={setSelectedRole} />
          </div>
          <div className="flex items-end">
            <Button type="submit" disabled={!canAdminMembers || isBusy || !selectedUserID}>
              <Plus data-icon="inline-start" />
              {isAdding ? t("Adding...") : t("Add member")}
            </Button>
          </div>
        </form>

        <div className="grid gap-2 rounded-md border p-3 md:grid-cols-[1fr_180px_auto]">
          <div className="relative">
            <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="pl-8"
              value={memberSearchQuery}
              placeholder={t("Search members")}
              onChange={(event) => setMemberSearchQuery(event.target.value)}
            />
          </div>
          <Select value={roleFilter} onValueChange={setRoleFilter}>
            <SelectTrigger>
              <SelectValue placeholder={t("Role")} />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="all">{t("All roles")}</SelectItem>
                {projectRoles.map((role) => (
                  <SelectItem key={role} value={role}>
                    {t(role)}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
            <RefreshCw data-icon="inline-start" />
            {t("Reload")}
          </Button>
        </div>

        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("Member")}</TableHead>
                <TableHead>{t("Source")}</TableHead>
                <TableHead>{t("Role")}</TableHead>
                <TableHead>{t("Change role")}</TableHead>
                <TableHead className="text-right">{t("Actions")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {membersQuery.query.isFetching && !membersQuery.query.data ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-muted-foreground">{t("Loading project members...")}</TableCell>
                </TableRow>
              ) : null}
              {!membersQuery.query.isFetching && filteredMembers.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-muted-foreground">{members.length === 0 ? t("No project-specific members yet.") : t("No members match the current filters.")}</TableCell>
                </TableRow>
              ) : null}
              {filteredMembers.map((member) => (
                <TableRow key={member.id || member.user_id}>
                  <TableCell>
                    <div className="flex flex-col gap-1">
                      <span className="font-medium">{member.display_name || member.username || `#${member.user_id}`}</span>
                      <span className="text-xs text-muted-foreground">
                        @{member.username || member.user_id} · {member.email || t("No email")}
                      </span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline">{member.source || "project"}</Badge>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-col gap-1">
                      <Badge variant="secondary" className="w-fit">{t(normalizeRole(member.role))}</Badge>
                      <span className="text-xs text-muted-foreground">{roleDescription(normalizeRole(member.role), t)}</span>
                    </div>
                  </TableCell>
                  <TableCell className="min-w-44">
                    <RoleSelect value={normalizeRole(member.role)} disabled={!canAdminMembers || isBusy} t={t} onChange={(role) => void submitUpdateMember(member, role)} />
                  </TableCell>
                  <TableCell>
                    <div className="flex justify-end">
                      <ConfirmAction
                        title={t("Remove project member?")}
                        description={t("This removes only the project-specific role. Organization membership is not changed.")}
                        confirmLabel={t("Remove")}
                        cancelLabel={t("Cancel")}
                        onConfirm={() => void submitDeleteMember(member)}
                      >
                        <Button type="button" size="sm" variant="outline" disabled={!canAdminMembers || isBusy}>
                          <Trash2 data-icon="inline-start" />
                          {isDeleting ? t("Removing...") : t("Remove")}
                        </Button>
                      </ConfirmAction>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
};

const MemberStat = ({ label, value }: { label: string; value: number }) => (
  <div className="rounded-md border bg-card p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
  </div>
);

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
      <SelectGroup>
        {projectRoles.map((role) => (
          <SelectItem key={role} value={role}>
            {t(role)}
          </SelectItem>
        ))}
      </SelectGroup>
    </SelectContent>
  </Select>
);

const normalizeRole = (value: string): string => (projectRoles.includes(value) ? value : "developer");

const roleDescription = (role: string, t: (text: string) => string): string => {
  switch (role) {
    case "guest":
      return t("Can view and comment.");
    case "reporter":
      return t("Can inspect code and issues.");
    case "developer":
      return t("Can push branches and open merge requests.");
    case "maintainer":
      return t("Can merge and manage repository settings.");
    case "owner":
      return t("Full project administration.");
    default:
      return t("Project member role.");
  }
};

const filterMembers = (members: RepositoryProjectMemberView[], query: string, role: string): RepositoryProjectMemberView[] => {
  const normalizedQuery = query.trim().toLowerCase();
  return members.filter((member) => {
    const matchesRole = role === "all" || normalizeRole(member.role) === role;
    if (!matchesRole) {
      return false;
    }
    if (!normalizedQuery) {
      return true;
    }
    return `${member.display_name ?? ""} ${member.username} ${member.email} ${member.user_id}`.toLowerCase().includes(normalizedQuery);
  });
};

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
