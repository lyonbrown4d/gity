import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { RepositoryBranchAccessLevel, RepositoryBranchProtectionPatch, RepositoryBranchView } from "@/pages/types";
import type { RepositoryPermissions } from "./repository-permissions";
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
  permissions: RepositoryPermissions;
  onChangeNewBranchName: (value: string) => void;
  onSubmitCreateBranch: (event: React.FormEvent<HTMLFormElement>) => void;
  onToggleBranchProtection: (branch: RepositoryBranchView, protect: boolean) => void;
  onUpdateBranchProtection: (branch: RepositoryBranchView, patch: RepositoryBranchProtectionPatch) => void;
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
  permissions,
  onChangeNewBranchName,
  onSubmitCreateBranch,
  onToggleBranchProtection,
  onUpdateBranchProtection,
  onDeleteBranch,
}: RepositoryBranchesTabProps): JSX.Element => {
  const canPushBranches = permissions.repositoryPush;
  const canAdminBranches = permissions.repositoryAdmin;

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Branches")}</CardTitle>
        <CardDescription>{t("Manage project branches and protections.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {!canPushBranches ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect branches, but cannot change them.")}</AlertDescription>
          </Alert>
        ) : null}

        <form className="grid gap-2 md:grid-cols-[1fr_auto]" onSubmit={onSubmitCreateBranch}>
          <Input
            placeholder={t("New branch name")}
            value={newBranchName}
            disabled={!canPushBranches}
            onChange={(event) => onChangeNewBranchName(event.target.value)}
          />
          <Button type="submit" disabled={!canPushBranches || isCreatingBranch || isUpdatingBranch}>
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
                  {protection ? (
                    <div className="mt-3 grid gap-3 rounded-md border bg-muted/10 p-3 lg:grid-cols-2">
                        <BranchAccessSelect
                          label={t("Push access")}
                          value={protection.push_access_level}
                          disabled={!canAdminBranches || isUpdatingBranch}
                          t={t}
                        onChange={(value) => onUpdateBranchProtection(branch, { push_access_level: value })}
                      />
                        <BranchAccessSelect
                          label={t("Merge access")}
                          value={protection.merge_access_level}
                          disabled={!canAdminBranches || isUpdatingBranch}
                          t={t}
                        onChange={(value) => onUpdateBranchProtection(branch, { merge_access_level: value })}
                      />
                      <div className="flex flex-wrap gap-2 lg:col-span-2">
                        <ProtectionToggle
                          label={t("Require merge request")}
                          enabled={protection.require_merge_request}
                          disabled={!canAdminBranches || isUpdatingBranch}
                          onClick={() => onUpdateBranchProtection(branch, { require_merge_request: !protection.require_merge_request })}
                        />
                        <ProtectionToggle
                          label={t("Require successful pipeline")}
                          enabled={protection.require_pipeline_success}
                          disabled={!canAdminBranches || isUpdatingBranch}
                          onClick={() => onUpdateBranchProtection(branch, { require_pipeline_success: !protection.require_pipeline_success })}
                        />
                        <ProtectionToggle
                          label={t("Allow force push")}
                          enabled={protection.allow_force_push}
                          disabled={!canAdminBranches || isUpdatingBranch}
                          onClick={() => onUpdateBranchProtection(branch, { allow_force_push: !protection.allow_force_push })}
                        />
                        <ProtectionToggle
                          label={t("Allow delete")}
                          enabled={protection.allow_delete}
                          disabled={!canAdminBranches || isUpdatingBranch}
                          onClick={() => onUpdateBranchProtection(branch, { allow_delete: !protection.allow_delete })}
                        />
                      </div>
                    </div>
                  ) : null}
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={!canAdminBranches || isUpdatingBranch || isDeletingBranch}
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
                    <Button type="button" size="sm" variant="destructive" disabled={!canAdminBranches || isDefault || isUpdatingBranch || isDeletingBranch}>
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

const ACCESS_LEVELS: RepositoryBranchAccessLevel[] = ["no_one", "developer", "maintainer", "owner"];

const BranchAccessSelect = ({
  label,
  value,
  disabled,
  t,
  onChange,
}: {
  label: string;
  value: RepositoryBranchAccessLevel;
  disabled: boolean;
  t: (text: string) => string;
  onChange: (value: RepositoryBranchAccessLevel) => void;
}) => (
  <div className="space-y-1">
    <p className="text-xs font-medium text-muted-foreground">{label}</p>
    <Select
      value={value}
      disabled={disabled}
      onValueChange={(nextValue) => {
        if (isAccessLevel(nextValue)) {
          onChange(nextValue);
        }
      }}
    >
      <SelectTrigger>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {ACCESS_LEVELS.map((level) => (
          <SelectItem key={level} value={level}>
            {t(accessLevelLabel(level))}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  </div>
);

const ProtectionToggle = ({
  label,
  enabled,
  disabled,
  onClick,
}: {
  label: string;
  enabled: boolean;
  disabled: boolean;
  onClick: () => void;
}) => (
  <Button type="button" size="sm" variant={enabled ? "default" : "outline"} disabled={disabled} onClick={onClick}>
    {label}
  </Button>
);

const isAccessLevel = (value: string): value is RepositoryBranchAccessLevel =>
  ACCESS_LEVELS.includes(value as RepositoryBranchAccessLevel);

const accessLevelLabel = (level: RepositoryBranchAccessLevel): string => {
  switch (level) {
    case "no_one":
      return "No one";
    case "developer":
      return "Developer";
    case "maintainer":
      return "Maintainer";
    case "owner":
      return "Owner";
  }
};
