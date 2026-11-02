import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/surface/section";
import type { RepositoryView } from "@/pages/types";
import { RepositoryCredentialsPanel } from "./repository-credentials-panel";
import { RepositoryMembersPanel } from "./repository-members-panel";
import type { RepositoryPermissions } from "./repository-permissions";

interface RepositorySettingsTabProps {
  repository: RepositoryView;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  isDeleting: boolean;
  onError: (message: string | null) => void;
  onDelete: (confirmation: string) => void;
}

export const RepositorySettingsTab = ({
  repository,
  permissions,
  t,
  isDeleting,
  onError,
  onDelete,
}: RepositorySettingsTabProps): JSX.Element => {
  const canDeleteProject = permissions.projectDelete;

  return (
    <div className="flex flex-col gap-4">
      <Section
        title={t("Settings")}
        description={t("Project metadata and danger zone.")}
      >
        <div className="flex flex-col gap-4">
          <dl className="divide-y divide-border border-y border-border">
            <Property label={t("Project UUID")}>
              <span className="break-all font-mono text-xs">{repository.uuid}</span>
            </Property>
            <Property label={t("Project key")}>{repository.key}</Property>
            <Property label={t("Visibility")}>{repository.visibility}</Property>
            <Property label={t("Default branch")}>{repository.default_branch}</Property>
          </dl>

          <div className="border border-destructive/40 bg-destructive/10 px-4 py-3">
            <p className="text-sm font-medium text-destructive">{t("Danger zone")}</p>
            <p className="mt-1 text-xs text-destructive/80">
              {t("Deleting a project is irreversible.")}
            </p>
            {!canDeleteProject ? (
              <Alert className="mt-3">
                <AlertDescription>{t("Only project owners can delete this project.")}</AlertDescription>
              </Alert>
            ) : null}
            <ConfirmAction
              title={t("Delete project \"{name}\"?").replace("{name}", repository.name)}
              description={t("This action cannot be undone.")}
              confirmLabel={t("Delete")}
              cancelLabel={t("Cancel")}
              verificationLabel={t("Type {path} to confirm deletion.").replace("{path}", repository.full_path)}
              verificationValue={repository.full_path}
              verificationPlaceholder={repository.full_path}
              onConfirm={(verification) => onDelete(verification ?? "")}
            >
              <Button type="button" variant="destructive" size="sm" className="mt-3" disabled={!canDeleteProject || isDeleting}>
                {t("Delete")}
              </Button>
            </ConfirmAction>
          </div>
        </div>
      </Section>

      <RepositoryMembersPanel
        repoId={repository.id}
        permissions={permissions}
        t={t}
        onError={onError}
      />

      <RepositoryCredentialsPanel
        repoId={repository.id}
        permissions={permissions}
        t={t}
        onError={onError}
      />
    </div>
  );
};

const Property = ({ label, children }: { label: string; children: React.ReactNode }) => (
  <div className="flex items-start gap-3 py-2">
    <dt className="w-28 shrink-0 text-xs text-muted-foreground">{label}</dt>
    <dd className="min-w-0 flex-1 text-sm">{children}</dd>
  </div>
);
