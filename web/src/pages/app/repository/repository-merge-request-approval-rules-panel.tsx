import { useEffect, useMemo, useState } from "react";
import { Pencil, Plus, RefreshCw, ShieldCheck, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { RepositoryBranchView, RepositoryMergeRequestApprovalRuleView, UserView } from "@/pages/types";
import { extractErrorMessage } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeBoolean, normalizeNumber, normalizeString, normalizeStringArray, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryMergeRequestApprovalRulesPanelProps {
  repoId: string;
  branches: RepositoryBranchView[];
  defaultBranch: string;
  users: UserView[];
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
  onRulesChanged?: () => void;
}

export const RepositoryMergeRequestApprovalRulesPanel = ({
  repoId,
  branches,
  defaultBranch,
  users,
  permissions,
  t,
  onError,
  onRulesChanged,
}: RepositoryMergeRequestApprovalRulesPanelProps): JSX.Element => {
  const rulesQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/merge-request-approval-rules`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: createRule, isLoading: isCreating } = useCustomMutation<RawRecord>();
  const { mutateAsync: updateRule, isLoading: isUpdating } = useCustomMutation<RawRecord>();
  const { mutateAsync: deleteRule, isLoading: isDeleting } = useCustomMutation<RawRecord>();
  const [editingRuleID, setEditingRuleID] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [targetBranch, setTargetBranch] = useState(defaultBranch || "*");
  const [approvalsRequired, setApprovalsRequired] = useState("1");
  const [eligibleUserIDs, setEligibleUserIDs] = useState<string[]>([]);
  const [draftUserID, setDraftUserID] = useState("");
  const [codeOwner, setCodeOwner] = useState(false);
  const rules = useMemo(
    () => resolveApprovalRules(rulesQuery.data?.data).map(normalizeApprovalRule),
    [rulesQuery.data?.data],
  );
  const userByID = useMemo(() => new Map(users.map((user) => [user.id, user])), [users]);
  const availableUsers = users.filter((user) => !eligibleUserIDs.includes(user.id));
  const canAdminRules = permissions.mergeRequestMerge;
  const isBusy = isCreating || isUpdating || isDeleting;

  const reload = async () => {
    const result = await rulesQuery.refetch();
    onError(result.error ? extractErrorMessage(result.error) : null);
    onRulesChanged?.();
  };

  const resetForm = () => {
    setEditingRuleID(null);
    setName("");
    setTargetBranch(defaultBranch || "*");
    setApprovalsRequired("1");
    setEligibleUserIDs([]);
    setDraftUserID("");
    setCodeOwner(false);
  };

  const editRule = (rule: RepositoryMergeRequestApprovalRuleView) => {
    setEditingRuleID(rule.id);
    setName(rule.name);
    setTargetBranch(rule.target_branch || "*");
    setApprovalsRequired(String(rule.approvals_required || 1));
    setEligibleUserIDs(rule.eligible_user_ids);
    setDraftUserID("");
    setCodeOwner(rule.code_owner);
  };

  const submitRule = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedName = name.trim();
    if (!normalizedName) {
      onError(t("Approval rule name is required."));
      return;
    }
    const required = Math.max(1, Number.parseInt(approvalsRequired, 10) || 1);
    onError(null);
    const values = {
      name: normalizedName,
      target_branch: targetBranch || "*",
      approvals_required: required,
      eligible_user_ids: eligibleUserIDs.map((item) => Number.parseInt(item, 10)).filter(Number.isFinite),
      code_owner: codeOwner,
    };
    try {
      if (editingRuleID) {
        await updateRule({
          url: `/projects/${repoId}/merge-request-approval-rules/${editingRuleID}`,
          method: "patch",
          values,
        });
      } else {
        await createRule({
          url: `/projects/${repoId}/merge-request-approval-rules`,
          method: "post",
          values,
        });
      }
      resetForm();
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitDeleteRule = async (rule: RepositoryMergeRequestApprovalRuleView) => {
    onError(null);
    try {
      await deleteRule({
        url: `/projects/${repoId}/merge-request-approval-rules/${rule.id}`,
        method: "delete",
        values: {},
      });
      if (editingRuleID === rule.id) {
        resetForm();
      }
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    if (rulesQuery.error) {
      onError(extractErrorMessage(rulesQuery.error));
    }
  }, [rulesQuery.error, onError]);

  return (
    <div className="rounded-md border p-3">
      <div className="mb-3 flex flex-wrap items-start justify-between gap-2">
        <div>
          <p className="flex items-center gap-2 font-medium">
            <ShieldCheck className="size-4" />
            {t("Approval rules")}
          </p>
          <p className="text-xs text-muted-foreground">{t("Configure branch scoped approvals and CODEOWNERS requirements.")}</p>
        </div>
        <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
          <RefreshCw className="size-4" />
          {t("Reload")}
        </Button>
      </div>

      {!canAdminRules ? (
        <Alert className="mb-3">
          <AlertDescription>{t("Your current project role can inspect approval rules, but cannot change them.")}</AlertDescription>
        </Alert>
      ) : null}

      <form className="space-y-3 rounded-md border bg-muted/10 p-3" onSubmit={submitRule}>
        <div className="grid gap-3 lg:grid-cols-[1fr_180px_140px]">
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground" htmlFor="approval-rule-name">
              {t("Rule name")}
            </Label>
            <Input
              id="approval-rule-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder={t("Maintainer approval")}
              disabled={!canAdminRules || isBusy}
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground">{t("Target branch")}</Label>
            <Select value={targetBranch || "*"} onValueChange={setTargetBranch} disabled={!canAdminRules || isBusy}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="*">{t("All branches")}</SelectItem>
                {branches.map((branch) => (
                  <SelectItem key={branch.name} value={branch.name}>
                    {branch.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1">
            <Label className="text-xs text-muted-foreground" htmlFor="approval-rule-required">
              {t("Required")}
            </Label>
            <Input
              id="approval-rule-required"
              value={approvalsRequired}
              onChange={(event) => setApprovalsRequired(event.target.value)}
              type="number"
              min={1}
              disabled={!canAdminRules || isBusy}
            />
          </div>
        </div>

        <div className="grid gap-3 lg:grid-cols-[1fr_auto]">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">{t("Eligible users")}</Label>
            <div className="flex flex-wrap gap-2">
              {eligibleUserIDs.length === 0 ? <span className="text-xs text-muted-foreground">{t("Any approver can satisfy this rule.")}</span> : null}
              {eligibleUserIDs.map((userID) => (
                <Badge key={userID} variant="outline" className="gap-2">
                  {formatUserLabel(userByID.get(userID), userID)}
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    className="h-4 px-1 text-muted-foreground hover:text-foreground"
                    disabled={!canAdminRules || isBusy}
                    onClick={() => setEligibleUserIDs((current) => current.filter((item) => item !== userID))}
                  >
                    x
                  </Button>
                </Badge>
              ))}
            </div>
            <div className="grid gap-2 sm:grid-cols-[1fr_auto]">
              <Select value={draftUserID} onValueChange={setDraftUserID} disabled={!canAdminRules || isBusy || availableUsers.length === 0}>
                <SelectTrigger>
                  <SelectValue placeholder={availableUsers.length === 0 ? t("No users available") : t("Select eligible user")} />
                </SelectTrigger>
                <SelectContent>
                  {availableUsers.map((user) => (
                    <SelectItem key={user.id} value={user.id}>
                      {formatUserLabel(user, user.id)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                type="button"
                variant="outline"
                disabled={!canAdminRules || isBusy || !draftUserID}
                onClick={() => {
                  if (!draftUserID) return;
                  setEligibleUserIDs((current) => [...current, draftUserID]);
                  setDraftUserID("");
                }}
              >
                {t("Add user")}
              </Button>
            </div>
          </div>

          <div className="flex flex-col justify-end gap-2">
            <Button type="button" variant={codeOwner ? "secondary" : "outline"} disabled={!canAdminRules || isBusy} onClick={() => setCodeOwner((current) => !current)}>
              {codeOwner ? t("CODEOWNERS required") : t("CODEOWNERS optional")}
            </Button>
            <Button type="submit" disabled={!canAdminRules || isBusy || !name.trim()}>
              <Plus className="size-4" />
              {editingRuleID ? (isUpdating ? t("Updating...") : t("Update rule")) : (isCreating ? t("Creating...") : t("Create rule"))}
            </Button>
            {editingRuleID ? (
              <Button type="button" variant="ghost" disabled={isBusy} onClick={resetForm}>
                {t("Cancel edit")}
              </Button>
            ) : null}
          </div>
        </div>
      </form>

      <div className="mt-3 space-y-2 rounded-md border p-2">
        {rulesQuery.isFetching && !rulesQuery.data ? (
          <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading approval rules...")}</p>
        ) : null}
        {rules.length === 0 ? (
          <p className="px-2 py-2 text-sm text-muted-foreground">{t("No approval rules configured.")}</p>
        ) : null}
        {rules.map((rule) => (
          <div key={rule.id} className="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3">
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <p className="font-medium">{rule.name}</p>
                <Badge variant="outline">{rule.target_branch || "*"}</Badge>
                <Badge variant="secondary">{rule.approvals_required} {t("approval(s)")}</Badge>
                {rule.code_owner ? <Badge>{t("CODEOWNERS")}</Badge> : null}
              </div>
              <p className="mt-1 text-xs text-muted-foreground">
                {rule.eligible_user_ids.length > 0
                  ? rule.eligible_user_ids.map((userID) => formatUserLabel(userByID.get(userID), userID)).join(", ")
                  : t("Any approver can satisfy this rule.")}
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button type="button" size="sm" variant="outline" disabled={!canAdminRules || isBusy} onClick={() => editRule(rule)}>
                <Pencil className="size-4" />
                {t("Edit")}
              </Button>
              <ConfirmAction
                title={t("Delete approval rule?")}
                description={t("Existing merge request approvals remain, but future checks will ignore this rule.")}
                confirmLabel={t("Delete")}
                cancelLabel={t("Cancel")}
                onConfirm={() => void submitDeleteRule(rule)}
              >
                <Button type="button" size="sm" variant="outline" disabled={!canAdminRules || isBusy}>
                  <Trash2 className="size-4" />
                  {isDeleting ? t("Deleting...") : t("Delete")}
                </Button>
              </ConfirmAction>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

const resolveApprovalRules = (payload: unknown): RawRecord[] => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : payload;
  if (Array.isArray(raw)) {
    return raw.filter(isRecord);
  }
  if (!isRecord(raw)) {
    return [];
  }
  return resolveRecordArray(raw.rules ?? raw.Rules);
};

const normalizeApprovalRule = (raw: RawRecord): RepositoryMergeRequestApprovalRuleView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  name: normalizeString(raw.name ?? raw.Name),
  target_branch: normalizeString(raw.target_branch ?? raw.TargetBranch) || "*",
  approvals_required: normalizeNumber(raw.approvals_required ?? raw.ApprovalsRequired) || 1,
  eligible_user_ids: normalizeStringArray(raw.eligible_user_ids ?? raw.EligibleUserIDs),
  code_owner: normalizeBoolean(raw.code_owner ?? raw.CodeOwner),
});

const formatUserLabel = (user: UserView | undefined, fallbackID: string): string => {
  if (!user) {
    return `#${fallbackID}`;
  }
  const displayName = user.display_name?.trim();
  return displayName ? `${displayName} (@${user.username})` : `@${user.username}`;
};
