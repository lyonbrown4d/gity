import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { RepositoryBranchView } from "@/pages/types";
import { shortSha } from "./repository-utils";

interface RepositoryBranchesTabProps {
  t: (text: string) => string;
  branches: RepositoryBranchView[];
  defaultBranch: string;
  newBranchName: string;
  isLoadingBranches: boolean;
  isCreatingBranch: boolean;
  isUpdatingBranch: boolean;
  isDeletingBranch: boolean;
  onChangeNewBranchName: (value: string) => void;
  onSubmitCreateBranch: (event: React.FormEvent<HTMLFormElement>) => void;
  onToggleBranchProtection: (branch: RepositoryBranchView, protect: boolean) => void;
  onDeleteBranch: (branch: RepositoryBranchView) => void;
}

export const RepositoryBranchesTab = ({
  t,
  branches,
  defaultBranch,
  newBranchName,
  isLoadingBranches,
  isCreatingBranch,
  isUpdatingBranch,
  isDeletingBranch,
  onChangeNewBranchName,
  onSubmitCreateBranch,
  onToggleBranchProtection,
  onDeleteBranch,
}: RepositoryBranchesTabProps): JSX.Element => {
  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Branches")}</CardTitle>
        <CardDescription>{t("Manage project branches and protections.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <form className="grid gap-2 md:grid-cols-[1fr_auto]" onSubmit={onSubmitCreateBranch}>
          <Input
            placeholder={t("New branch name")}
            value={newBranchName}
            onChange={(event) => onChangeNewBranchName(event.target.value)}
          />
          <Button type="submit" disabled={isCreatingBranch || isUpdatingBranch}>
            {isCreatingBranch ? t("Creating...") : t("Create branch")}
          </Button>
        </form>

        {isLoadingBranches ? <p className="text-sm text-muted-foreground">{t("Loading branches...")}</p> : null}

        <div className="space-y-2">
          {branches.map((branch) => {
            const isDefault = branch.name === defaultBranch;
            const protection = branch.protection;
            return (
              <div key={branch.name} className="flex flex-col gap-3 rounded-md border p-3 md:flex-row md:items-center md:justify-between">
                <div className="min-w-0 space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="font-medium">{branch.name}</p>
                    {isDefault ? <Badge variant="secondary">{t("Default")}</Badge> : null}
                    <Badge variant={branch.is_protected ? "default" : "outline"}>
                      {branch.is_protected ? t("Protected") : t("Unprotected")}
                    </Badge>
                  </div>
                  <p className="truncate text-xs text-muted-foreground">
                    {t("Last commit")}: {branch.last_commit_sha ? shortSha(branch.last_commit_sha) : t("N/A")}
                  </p>
                  {protection ? (
                    <p className="text-xs text-muted-foreground">
                      {t("Push")}: {t(protection.push_access_level)} / {t("Merge")}: {t(protection.merge_access_level)}
                      {protection.allow_delete ? ` / ${t("Deletion allowed")}` : ` / ${t("Deletion blocked")}`}
                    </p>
                  ) : null}
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={isUpdatingBranch || isDeletingBranch}
                    onClick={() => onToggleBranchProtection(branch, !branch.is_protected)}
                  >
                    {branch.is_protected ? t("Unprotect") : t("Protect")}
                  </Button>
                  <ConfirmAction
                    title={t("Delete branch \"{name}\"?").replace("{name}", branch.name)}
                    description={isDefault ? t("Default branch cannot be deleted.") : t("This action cannot be undone.")}
                    confirmLabel={t("Delete")}
                    cancelLabel={t("Cancel")}
                    verificationLabel={t("Type {name} to confirm branch deletion.").replace("{name}", branch.name)}
                    verificationValue={branch.name}
                    onConfirm={() => onDeleteBranch(branch)}
                  >
                    <Button type="button" size="sm" variant="destructive" disabled={isDefault || isUpdatingBranch || isDeletingBranch}>
                      {t("Delete")}
                    </Button>
                  </ConfirmAction>
                </div>
              </div>
            );
          })}
          {!isLoadingBranches && branches.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("No branches found.")}</p>
          ) : null}
        </div>
      </CardContent>
    </Card>
  );
};
