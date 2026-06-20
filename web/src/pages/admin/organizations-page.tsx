import { useEffect, useMemo, useState } from "react";
import { useCreate, useDelete, useList, useUpdate } from "@refinedev/core";
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
import type { OrganizationMemberView, OrganizationView, RepositoryView } from "@/pages/types";

export function AdminOrganizationsPage(): JSX.Element {
  const { t } = useI18n();
  const [selectedOrg, setSelectedOrg] = useState<string>("");
  const [actionError, setActionError] = useState<string | null>(null);

  const [isCreateModalOpen, setCreateModalOpen] = useState(false);
  const [createKey, setCreateKey] = useState("");
  const [createName, setCreateName] = useState("");

  const [isEditModalOpen, setEditModalOpen] = useState(false);
  const [editKey, setEditKey] = useState("");
  const [editName, setEditName] = useState("");

  const [isMemberModalOpen, setMemberModalOpen] = useState(false);
  const [memberUserId, setMemberUserId] = useState("");
  const [memberRole, setMemberRole] = useState("member");

  const { mutate: createOrganization, mutation: { isPending: isCreatingOrg } } = useCreate<OrganizationView>();
  const { mutate: updateOrganization, mutation: { isPending: isUpdatingOrg } } = useUpdate<OrganizationView>();
  const { mutate: deleteOrganization, mutation: { isPending: isDeletingOrg } } = useDelete<OrganizationView>();
  const { mutate: addOrganizationMember, mutation: { isPending: isAddingMember } } =
    useCreate<OrganizationMemberView>();

  const orgQuery = useList<OrganizationView>({
    resource: "organizations",
  });
  const orgs = orgQuery.result.data ?? [];

  useEffect(() => {
    if (!selectedOrg && orgs[0]) {
      setSelectedOrg(orgs[0].id);
    }
  }, [orgs, selectedOrg]);

  const selectedOrgModel = useMemo(
    () => orgs.find((org) => org.id === selectedOrg) ?? null,
    [orgs, selectedOrg],
  );

  const membersQuery = useList<OrganizationMemberView>({
    resource: "organization-members",
    meta: {
      organization_id: selectedOrg,
    },
    queryOptions: {
      enabled: Boolean(selectedOrg),
    },
  });
  const members = membersQuery.result.data ?? [];

  const reposQuery = useList<RepositoryView>({
    resource: "projects",
    meta: {
      organization_id: selectedOrg,
    },
    queryOptions: {
      enabled: Boolean(selectedOrg),
    },
  });
  const repos = reposQuery.result.data ?? [];

  const isLoading = orgQuery.query.isLoading || membersQuery.query.isLoading || reposQuery.query.isLoading;
  const errorMessage = actionError
    ?? (orgQuery.query.error instanceof Error
      ? orgQuery.query.error.message
      : membersQuery.query.error instanceof Error
        ? membersQuery.query.error.message
        : reposQuery.query.error instanceof Error
          ? reposQuery.query.error.message
          : null);

  const closeCreateModal = () => {
    setCreateModalOpen(false);
    setCreateKey("");
    setCreateName("");
  };

  const closeEditModal = () => {
    setEditModalOpen(false);
    setEditKey("");
    setEditName("");
  };

  const closeMemberModal = () => {
    setMemberModalOpen(false);
    setMemberUserId("");
    setMemberRole("member");
  };

  const openEditModal = () => {
    if (!selectedOrgModel) {
      return;
    }
    setEditKey(selectedOrgModel.key);
    setEditName(selectedOrgModel.name);
    setEditModalOpen(true);
  };

  const submitCreateOrganization = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);

    createOrganization(
      {
        resource: "organizations",
        values: {
          key: createKey,
          name: createName,
        },
      },
      {
        onSuccess: async (result) => {
          closeCreateModal();
          await orgQuery.query.refetch();
          if (result.data?.id) {
            setSelectedOrg(String(result.data.id));
          }
        },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to create organization"));
        },
      },
    );
  };

  const submitUpdateOrganization = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedOrgModel) {
      setActionError(t("Please select an organization first."));
      return;
    }
    setActionError(null);

    updateOrganization(
      {
        resource: "organizations",
        id: selectedOrgModel.id,
        values: {
          key: editKey,
          name: editName,
        },
      },
      {
        onSuccess: async () => {
          closeEditModal();
          await orgQuery.query.refetch();
        },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to update organization"));
        },
      },
    );
  };

  const submitDeleteOrganization = () => {
    if (!selectedOrgModel) {
      setActionError(t("Please select an organization first."));
      return;
    }
    setActionError(null);
    deleteOrganization(
      {
        resource: "organizations",
        id: selectedOrgModel.id,
      },
      {
        onSuccess: async () => {
          const refreshed = await orgQuery.query.refetch();
          const next = refreshed.data?.data.find((org) => org.id !== selectedOrgModel.id);
          setSelectedOrg(next?.id ?? "");
          await membersQuery.query.refetch();
          await reposQuery.query.refetch();
        },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to delete organization"));
        },
      },
    );
  };

  const submitAddMember = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);
    if (!selectedOrg) {
      setActionError(t("Please select an organization first."));
      return;
    }

    addOrganizationMember(
      {
        resource: "organization-members",
        values: {
          organization_id: selectedOrg,
          user_id: memberUserId,
          role: memberRole,
        },
      },
      {
        onSuccess: async () => {
          closeMemberModal();
          await membersQuery.query.refetch();
        },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to add member"));
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
              <CardTitle>{t("Organizations")}</CardTitle>
              <CardDescription>
                {t("Full CRUD for organizations and member management entry.")}
              </CardDescription>
            </div>
            <Button type="button" onClick={() => setCreateModalOpen(true)} className="action-pop">
              {t("New Organization")}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-3 md:grid-cols-[240px_1fr] md:items-end">
            <div className="space-y-2">
              <Label htmlFor="selected-org">{t("Organization")}</Label>
              <Select value={selectedOrg} onValueChange={setSelectedOrg}>
                <SelectTrigger id="selected-org">
                  <SelectValue placeholder={t("Organization")} />
                </SelectTrigger>
                <SelectContent>
                  {orgs.map((org) => (
                    <SelectItem key={org.id} value={org.id}>
                      {org.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              {selectedOrgModel ? (
                <>
                  <Badge variant="secondary">{selectedOrgModel.name}</Badge>
                  <Badge variant="outline">{selectedOrgModel.key}</Badge>
                  <Badge variant="outline">{selectedOrgModel.role}</Badge>
                </>
              ) : null}
            </div>
          </div>

          <div className="flex flex-wrap gap-2">
            <Button type="button" variant="outline" onClick={openEditModal} disabled={!selectedOrgModel}>
              {t("Edit Organization")}
            </Button>
            <ConfirmAction
              title={selectedOrgModel ? t("Delete organization \"{name}\"?").replace("{name}", selectedOrgModel.name) : t("Delete Organization")}
              description={t("This action cannot be undone.")}
              confirmLabel={t("Delete")}
              cancelLabel={t("Cancel")}
              onConfirm={submitDeleteOrganization}
            >
              <Button
                type="button"
                variant="destructive"
                disabled={!selectedOrgModel || isDeletingOrg}
              >
                {isDeletingOrg ? `${t("Delete")}...` : t("Delete Organization")}
              </Button>
            </ConfirmAction>
            <Button type="button" variant="secondary" onClick={() => setMemberModalOpen(true)} disabled={!selectedOrg}>
              {t("Add Member")}
            </Button>
          </div>

          {errorMessage ? (
            <Alert variant="destructive">
              <AlertDescription>{errorMessage}</AlertDescription>
            </Alert>
          ) : null}
        </CardContent>
      </Card>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("Members")}</CardTitle>
            <CardDescription>{t("Members of the selected organization.")}</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="space-y-2">
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            ) : (
              <div className="space-y-2">
                {members.map((member) => (
                  <div key={`${member.organization_id}-${member.user_id}`} className="rounded-md border p-2">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium">{member.username}</p>
                      <Badge variant="secondary">{member.role}</Badge>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {member.email} · {member.user_id}
                    </p>
                  </div>
                ))}
                {members.length === 0 ? (
                  <p className="text-sm text-muted-foreground">{t("No members found.")}</p>
                ) : null}
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("Projects")}</CardTitle>
            <CardDescription>{t("Projects under the selected organization.")}</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="space-y-2">
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            ) : (
              <div className="space-y-2">
                {repos.map((repo) => (
                  <div key={repo.id} className="rounded-md border p-2">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium">{repo.name}</p>
                      <Badge>{repo.visibility}</Badge>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {repo.key} · {repo.default_branch}
                    </p>
                  </div>
                ))}
                {repos.length === 0 ? (
                  <p className="text-sm text-muted-foreground">{t("No projects found.")}</p>
                ) : null}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Modal open={isCreateModalOpen} onClose={closeCreateModal} title={t("Create Organization")}>
        <form className="grid gap-3" onSubmit={submitCreateOrganization}>
          <div className="space-y-2">
            <Label htmlFor="create-org-key">{t("Organization key")}</Label>
            <Input
              id="create-org-key"
              value={createKey}
              onChange={(event) => setCreateKey(event.target.value)}
              placeholder="acme"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-org-name">{t("Organization name")}</Label>
            <Input
              id="create-org-name"
              value={createName}
              onChange={(event) => setCreateName(event.target.value)}
              placeholder="Acme"
              required
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={closeCreateModal}>
              {t("Cancel")}
            </Button>
            <Button type="submit" disabled={isCreatingOrg}>
              {isCreatingOrg ? t("Creating...") : t("Create")}
            </Button>
          </div>
        </form>
      </Modal>

      <Modal open={isEditModalOpen} onClose={closeEditModal} title={t("Edit Organization")}>
        <form className="grid gap-3" onSubmit={submitUpdateOrganization}>
          <div className="space-y-2">
            <Label htmlFor="edit-org-key">{t("Organization key")}</Label>
            <Input
              id="edit-org-key"
              value={editKey}
              onChange={(event) => setEditKey(event.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-org-name">{t("Organization name")}</Label>
            <Input
              id="edit-org-name"
              value={editName}
              onChange={(event) => setEditName(event.target.value)}
              required
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={closeEditModal}>
              {t("Cancel")}
            </Button>
            <Button type="submit" disabled={isUpdatingOrg}>
              {isUpdatingOrg ? `${t("Save")}...` : t("Save")}
            </Button>
          </div>
        </form>
      </Modal>

      <Modal open={isMemberModalOpen} onClose={closeMemberModal} title={t("Add Organization Member")}>
        <form className="grid gap-3" onSubmit={submitAddMember}>
          <div className="space-y-2">
            <Label htmlFor="member-user-id">{t("User ID")}</Label>
            <Input
              id="member-user-id"
              value={memberUserId}
              onChange={(event) => setMemberUserId(event.target.value)}
              placeholder="user-id"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="member-role">{t("Role")}</Label>
            <Select value={memberRole} onValueChange={setMemberRole}>
              <SelectTrigger id="member-role">
                <SelectValue placeholder={t("Role")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="member">{t("member")}</SelectItem>
                <SelectItem value="owner">{t("owner")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={closeMemberModal}>
              {t("Cancel")}
            </Button>
            <Button type="submit" disabled={isAddingMember}>
              {isAddingMember ? t("Adding...") : t("Add Member")}
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
