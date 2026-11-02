import { FormDialog as Modal } from "@/components/common/form-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useI18n } from "@/lib/i18n";
import type { UserAdminController } from "./use-user-admin";

export function UserDialogs({ controller }: { controller: UserAdminController }): JSX.Element {
  const { t } = useI18n();
  const c = controller;

  return (
    <>
      <Modal open={c.createForm.open} onClose={c.closeCreate} title={t("Create User")}>
        <form className="grid gap-3" onSubmit={c.submitCreate}>
          <div className="space-y-2">
            <Label htmlFor="create-user-username">{t("Username")}</Label>
            <Input
              id="create-user-username"
              value={c.createForm.username}
              onChange={(event) => c.setCreateForm({ ...c.createForm, username: event.target.value })}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-user-email">{t("Email")}</Label>
            <Input
              id="create-user-email"
              type="email"
              value={c.createForm.email}
              onChange={(event) => c.setCreateForm({ ...c.createForm, email: event.target.value })}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="create-user-password">{t("Password")}</Label>
            <Input
              id="create-user-password"
              type="password"
              value={c.createForm.password}
              onChange={(event) => c.setCreateForm({ ...c.createForm, password: event.target.value })}
              required
            />
          </div>
          <RoleSelect
            id="create-user-role"
            value={c.createForm.role}
            onChange={(role) => c.setCreateForm({ ...c.createForm, role })}
          />
          <div className="flex items-center justify-end gap-2">
            <Button type="button" variant="outline" onClick={c.closeCreate}>{t("Cancel")}</Button>
            <Button type="submit" disabled={c.isCreating}>
              {c.isCreating ? t("Creating...") : t("Create")}
            </Button>
          </div>
        </form>
      </Modal>

      <Modal open={c.editForm.open} onClose={c.closeEdit} title={t("Edit User")}>
        <form className="grid gap-3" onSubmit={c.submitUpdate}>
          <div className="space-y-2">
            <Label htmlFor="edit-user-username">{t("Username")}</Label>
            <Input
              id="edit-user-username"
              value={c.editForm.username}
              onChange={(event) => c.setEditForm({ ...c.editForm, username: event.target.value })}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-user-email">{t("Email")}</Label>
            <Input
              id="edit-user-email"
              type="email"
              value={c.editForm.email}
              onChange={(event) => c.setEditForm({ ...c.editForm, email: event.target.value })}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-user-password">{t("Password (optional)")}</Label>
            <Input
              id="edit-user-password"
              type="password"
              value={c.editForm.password}
              onChange={(event) => c.setEditForm({ ...c.editForm, password: event.target.value })}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-user-status">{t("Status")}</Label>
            <Select
              value={c.editForm.status}
              onValueChange={(value) => c.setEditForm({
                ...c.editForm,
                status: value === "disabled" ? "disabled" : "active",
              })}
            >
              <SelectTrigger id="edit-user-status"><SelectValue placeholder={t("Status")} /></SelectTrigger>
              <SelectContent>
                <SelectItem value="active">{t("active")}</SelectItem>
                <SelectItem value="disabled">{t("disabled")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <RoleSelect
            id="edit-user-role"
            value={c.editForm.role}
            onChange={(role) => c.setEditForm({ ...c.editForm, role })}
          />
          <div className="flex items-center justify-end gap-2">
            <Button type="button" variant="outline" onClick={c.closeEdit}>{t("Cancel")}</Button>
            <Button type="submit" disabled={c.isUpdating}>
              {c.isUpdating ? t("Saving...") : t("Save")}
            </Button>
          </div>
        </form>
      </Modal>
    </>
  );
}

interface RoleSelectProps {
  id: string;
  value: "user" | "super-admin";
  onChange: (value: "user" | "super-admin") => void;
}

function RoleSelect({ id, value, onChange }: RoleSelectProps): JSX.Element {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <Label htmlFor={id}>{t("Privilege level")}</Label>
      <Select value={value} onValueChange={(next) => onChange(next === "super-admin" ? "super-admin" : "user")}>
        <SelectTrigger id={id}><SelectValue placeholder={t("Privilege level")} /></SelectTrigger>
        <SelectContent>
          <SelectItem value="user">{t("Regular user")}</SelectItem>
          <SelectItem value="super-admin">{t("Super Admin")}</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}
