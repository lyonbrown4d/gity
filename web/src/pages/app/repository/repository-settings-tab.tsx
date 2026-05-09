import { ConfirmAction } from "@/components/common/confirm-action";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import type { RepositoryView } from "@/pages/types";

interface RepositorySettingsTabProps {
  repository: RepositoryView;
  t: (text: string) => string;
  isDeleting: boolean;
  onDelete: () => void;
}

export const RepositorySettingsTab = ({
  repository,
  t,
  isDeleting,
  onDelete,
}: RepositorySettingsTabProps): JSX.Element => {
  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Settings")}</CardTitle>
        <CardDescription>{t("Project metadata and danger zone.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-md border p-3 text-sm">
          <p>
            <span className="text-muted-foreground">{t("Project UUID")}:</span>{" "}
            <span className="font-mono text-xs">{repository.uuid}</span>
          </p>
          <p>
            <span className="text-muted-foreground">{t("Project key")}:</span> {repository.key}
          </p>
          <p>
            <span className="text-muted-foreground">{t("Visibility")}:</span> {repository.visibility}
          </p>
          <p>
            <span className="text-muted-foreground">{t("Default branch")}:</span> {repository.default_branch}
          </p>
        </div>

        <div className="rounded-md border border-destructive/40 bg-destructive/10 p-3">
          <p className="text-sm font-medium text-destructive">{t("Danger zone")}</p>
          <p className="mt-1 text-xs text-destructive/80">
            {t("Deleting a project is irreversible.")}
          </p>
          <ConfirmAction
            title={t("Delete project \"{name}\"?").replace("{name}", repository.name)}
            description={t("This action cannot be undone.")}
            confirmLabel={t("Delete")}
            cancelLabel={t("Cancel")}
            onConfirm={onDelete}
          >
            <Button type="button" variant="destructive" size="sm" className="mt-3" disabled={isDeleting}>
              {t("Delete")}
            </Button>
          </ConfirmAction>
        </div>
      </CardContent>
    </Card>
  );
};
