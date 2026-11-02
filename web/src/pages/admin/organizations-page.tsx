import { Plus, Settings2, UserPlus } from "lucide-react";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useI18n } from "@/lib/i18n";
import { ConsolePageHeader, MetricStrip } from "./workspace/console-primitives";
import { OrganizationDialogs } from "./workspace/organization-dialogs";
import { OrganizationPanels } from "./workspace/organization-panels";
import { useOrganizationAdmin } from "./workspace/use-organization-admin";

export function AdminOrganizationsPage(): JSX.Element {
  const { t } = useI18n();
  const c = useOrganizationAdmin();

  return (
    <div className="space-y-4 page-enter">
      <ConsolePageHeader
        eyebrow={t("Admin Side")}
        title={t("Organizations")}
        description={t("Full CRUD for organizations and member management entry.")}
        actions={(
          <Button
            type="button"
            size="sm"
            className="h-8"
            onClick={() => c.setCreateForm({ ...c.createForm, open: true })}
          >
            <Plus className="size-3.5" />
            {t("New Organization")}
          </Button>
        )}
      />

      <section className="rounded-md border border-border/80 bg-card">
        <div className="grid gap-3 p-3 lg:grid-cols-[minmax(220px,320px)_1fr_auto] lg:items-end">
          <div className="space-y-1.5">
            <Label htmlFor="selected-org" className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
              {t("Organization")}
            </Label>
            <Select value={c.selectedOrg} onValueChange={c.setSelectedOrg}>
              <SelectTrigger id="selected-org" className="h-8 text-xs">
                <SelectValue placeholder={t("Organization")} />
              </SelectTrigger>
              <SelectContent>
                {c.orgs.map((org) => <SelectItem key={org.id} value={org.id}>{org.name}</SelectItem>)}
              </SelectContent>
            </Select>
          </div>
          <div className="flex min-w-0 flex-wrap items-center gap-1.5 pb-0.5">
            {c.selectedOrgModel ? (
              <>
                <Badge variant="secondary">{c.selectedOrgModel.name}</Badge>
                <Badge variant="outline" className="font-mono">{c.selectedOrgModel.key}</Badge>
                <Badge variant="outline">{c.selectedOrgModel.role}</Badge>
              </>
            ) : <span className="text-xs text-muted-foreground">{t("No organizations available.")}</span>}
          </div>
          <div className="flex flex-wrap items-center gap-1.5">
            <Button type="button" variant="outline" size="sm" className="h-8" onClick={c.openEdit} disabled={!c.selectedOrgModel}>
              <Settings2 className="size-3.5" />{t("Edit Organization")}
            </Button>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              className="h-8"
              onClick={() => c.setMemberForm({ ...c.memberForm, open: true })}
              disabled={!c.selectedOrg}
            >
              <UserPlus className="size-3.5" />{t("Add Member")}
            </Button>
            <ConfirmAction
              title={c.selectedOrgModel ? t("Delete organization \"{name}\"?").replace("{name}", c.selectedOrgModel.name) : t("Delete Organization")}
              description={t("This action cannot be undone.")}
              confirmLabel={t("Delete")}
              cancelLabel={t("Cancel")}
              onConfirm={c.submitDelete}
            >
              <Button type="button" variant="destructive" size="sm" className="h-8" disabled={!c.selectedOrgModel || c.isDeleting}>
                {c.isDeleting ? `${t("Delete")}...` : t("Delete Organization")}
              </Button>
            </ConfirmAction>
          </div>
        </div>
      </section>

      {c.errorMessage ? <Alert variant="destructive"><AlertDescription>{c.errorMessage}</AlertDescription></Alert> : null}

      <MetricStrip items={[
        { label: t("Organizations"), value: c.orgs.length, detail: t("total"), active: c.orgs.length > 0 },
        { label: t("Members"), value: c.members.length, detail: t("member"), active: c.members.length > 0 },
        { label: t("Projects"), value: c.repos.length, detail: t("projects"), active: c.repos.length > 0 },
      ]} />

      <OrganizationPanels members={c.members} repos={c.repos} isLoading={c.isLoading} />
      <OrganizationDialogs controller={c} />
    </div>
  );
}
