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
        <CardDescription>{t("Repository metadata and danger zone.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-md border p-3 text-sm">
          <p>
            <span className="text-muted-foreground">{t("Repository key")}:</span> {repository.key}
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
            {t("Deleting a repository is irreversible.")}
          </p>
          <Button type="button" variant="destructive" size="sm" className="mt-3" disabled={isDeleting} onClick={onDelete}>
            {t("Delete")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};
