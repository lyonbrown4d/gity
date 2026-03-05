import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { RepositoryBranchView } from "@/pages/types";
import { shortSha } from "./repository-utils";

interface RepositoryBranchesTabProps {
  t: (text: string) => string;
  branches: RepositoryBranchView[];
  newBranchName: string;
  isLoadingBranches: boolean;
  isCreatingBranch: boolean;
  isUpdatingBranch: boolean;
  onChangeNewBranchName: (value: string) => void;
  onSubmitCreateBranch: (event: React.FormEvent<HTMLFormElement>) => void;
  onToggleBranchProtection: (branch: RepositoryBranchView, protect: boolean) => void;
}

export function RepositoryBranchesTab({
  t,
  branches,
  newBranchName,
  isLoadingBranches,
  isCreatingBranch,
  isUpdatingBranch,
  onChangeNewBranchName,
  onSubmitCreateBranch,
  onToggleBranchProtection,
}: RepositoryBranchesTabProps): JSX.Element {
  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Branches")}</CardTitle>
        <CardDescription>{t("Manage repository branches and protections.")}</CardDescription>
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
          {branches.map((branch) => (
            <div key={branch.name} className="flex flex-col gap-3 rounded-md border p-3 md:flex-row md:items-center md:justify-between">
              <div className="min-w-0">
                <p className="font-medium">{branch.name}</p>
                <p className="truncate text-xs text-muted-foreground">
                  {t("Last commit")}: {branch.last_commit_sha ? shortSha(branch.last_commit_sha) : t("N/A")}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <Badge variant={branch.is_protected ? "default" : "outline"}>
                  {branch.is_protected ? t("Protected") : t("Unprotected")}
                </Badge>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={isUpdatingBranch}
                  onClick={() => onToggleBranchProtection(branch, !branch.is_protected)}
                >
                  {branch.is_protected ? t("Unprotect") : t("Protect")}
                </Button>
              </div>
            </div>
          ))}
          {!isLoadingBranches && branches.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("No branches found.")}</p>
          ) : null}
        </div>
      </CardContent>
    </Card>
  );
}
