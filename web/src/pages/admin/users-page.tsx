import { Plus } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/lib/i18n";
import { ConsolePageHeader, MetricStrip } from "./workspace/console-primitives";
import { UserDialogs } from "./workspace/user-dialogs";
import { UserDirectory } from "./workspace/user-directory";
import { useUserAdmin } from "./workspace/use-user-admin";

export const AdminUsersPage = (): JSX.Element => {
  const { t } = useI18n();
  const c = useUserAdmin();

  return (
    <div className="space-y-4 page-enter">
      <ConsolePageHeader
        eyebrow={t("Admin Side")}
        title={t("Users")}
        description={t("User accounts in current admin scope.")}
        actions={(
          <Button
            type="button"
            size="sm"
            className="h-8"
            onClick={() => c.setCreateForm({ ...c.createForm, open: true })}
          >
            <Plus className="size-3.5" />
            {t("New User")}
          </Button>
        )}
      />

      {c.errorMessage ? <Alert variant="destructive"><AlertDescription>{c.errorMessage}</AlertDescription></Alert> : null}

      <MetricStrip items={[
        { label: t("Users"), value: c.totalUsers, detail: t("total"), active: c.totalUsers > 0 },
        { label: t("Status"), value: c.visibleStats.activeCount, detail: `${t("active")} (page)`, active: c.visibleStats.activeCount > 0 },
        { label: t("Privilege level"), value: c.visibleStats.superAdminCount, detail: `${t("super-admin")} (page)`, active: c.visibleStats.superAdminCount > 0 },
      ]} />

      <UserDirectory controller={c} />
      <UserDialogs controller={c} />
    </div>
  );
};
