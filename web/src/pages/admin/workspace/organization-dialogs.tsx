import { FormDialog as Modal } from "@/components/common/form-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useI18n } from "@/lib/i18n";
import type { OrganizationAdminController } from "./use-organization-admin";

export function OrganizationDialogs({ controller }: { controller: OrganizationAdminController }): JSX.Element {
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

      <Modal open={c.memberForm.open} onClose={c.closeMember} title={t("Add Organization Member")}>
        <form className="grid gap-3" onSubmit={c.submitMember}>
          <div className="space-y-2">
            <Label htmlFor="member-user-id">{t("User ID")}</Label>
            <Input
              id="member-user-id"
              value={c.memberForm.userId}
              onChange={(event) => c.setMemberForm({ ...c.memberForm, userId: event.target.value })}
              placeholder="user-id"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="member-role">{t("Role")}</Label>
            <Select
              value={c.memberForm.role}
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
            <Button type="button" variant="outline" onClick={c.closeMember}>{t("Cancel")}</Button>
            <Button type="submit" disabled={c.isAddingMember}>
              {c.isAddingMember ? t("Adding...") : t("Add Member")}
            </Button>
          </div>
        </form>
      </Modal>
    </>
  );
}
