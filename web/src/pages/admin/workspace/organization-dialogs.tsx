import { FormDialog as Modal } from "@/components/common/form-dialog";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useI18n } from "@/lib/i18n";
import { OrganizationMemberUserPicker } from "./organization-member-user-picker";
import type { OrganizationAdminController } from "./use-organization-admin";

export const OrganizationDialogs = ({ controller }: { controller: OrganizationAdminController }): JSX.Element => {
  const { t } = useI18n();
  const c = controller;

  return (
    <>
      <Modal open={c.createForm.open} onClose={c.closeCreate} title={t("Create Organization")}>
        <form className="grid gap-3" onSubmit={c.submitCreate}>
          <div className="space-y-2">
            <Label htmlFor="create-org-key">{t("Organization key")}</Label>
            <Input
              id="create-org-key"
              value={c.createForm.key}
              onChange={(event) => c.setCreateForm({ ...c.createForm, key: event.target.value })}
              placeholder="acme"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-org-name">{t("Organization name")}</Label>
            <Input
              id="create-org-name"
              value={c.createForm.name}
              onChange={(event) => c.setCreateForm({ ...c.createForm, name: event.target.value })}
              placeholder="Acme"
              required
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={c.closeCreate}>{t("Cancel")}</Button>
            <Button type="submit" disabled={c.isCreating}>
              {c.isCreating ? t("Creating...") : t("Create")}
            </Button>
          </div>
        </form>
      </Modal>

      <Modal open={c.editForm.open} onClose={c.closeEdit} title={t("Edit Organization")}>
        <form className="grid gap-3" onSubmit={c.submitUpdate}>
          <div className="space-y-2">
            <Label htmlFor="edit-org-key">{t("Organization key")}</Label>
            <Input
              id="edit-org-key"
              value={c.editForm.key}
              onChange={(event) => c.setEditForm({ ...c.editForm, key: event.target.value })}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-org-name">{t("Organization name")}</Label>
            <Input
              id="edit-org-name"
              value={c.editForm.name}
              onChange={(event) => c.setEditForm({ ...c.editForm, name: event.target.value })}
              required
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={c.closeEdit}>{t("Cancel")}</Button>
            <Button type="submit" disabled={c.isUpdating}>
              {c.isUpdating ? `${t("Save")}...` : t("Save")}
            </Button>
          </div>
        </form>
      </Modal>

      <Modal open={c.memberForm.open} onClose={c.closeMember} busy={c.isAddingMember} title={t("Add Organization Member")}>
        <form className="grid gap-3" onSubmit={c.submitMember}>
          {c.isLoadingUsers ? <p role="status" className="text-sm text-muted-foreground">{t("Loading...")}</p> : null}
          {c.usersError ? <Alert variant="destructive" role="alert"><AlertDescription className="flex items-center justify-between gap-3">
            <span>{t("Users could not be loaded.")}</span>
            <Button type="button" size="sm" variant="outline" disabled={c.isLoadingUsers} onClick={() => void c.retryUsers()}>{t("Retry")}</Button>
          </AlertDescription></Alert> : null}
          {c.memberError ? <Alert variant="destructive" role="alert"><AlertDescription>{c.memberError}</AlertDescription></Alert> : null}
          <div className="space-y-2">
            <Label>{t("User")}</Label>
            <OrganizationMemberUserPicker users={c.users} loading={c.isLoadingUsers} value={c.memberForm.userId}
              disabled={c.isAddingMember || Boolean(c.usersError)} t={t}
              onChange={(userId) => c.setMemberForm({ ...c.memberForm, userId })} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="member-role">{t("Role")}</Label>
            <Select
              value={c.memberForm.role}
              disabled={c.isAddingMember}
              onValueChange={(role) => c.setMemberForm({ ...c.memberForm, role })}
            >
              <SelectTrigger id="member-role"><SelectValue placeholder={t("Role")} /></SelectTrigger>
              <SelectContent>
                <SelectItem value="member">{t("member")}</SelectItem>
                <SelectItem value="owner">{t("owner")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" disabled={c.isAddingMember} onClick={c.closeMember}>{t("Cancel")}</Button>
            <Button type="submit" disabled={c.isAddingMember || !c.memberForm.userId}>
              {c.isAddingMember ? t("Adding...") : t("Add Member")}
            </Button>
          </div>
        </form>
      </Modal>
    </>
  );
};
