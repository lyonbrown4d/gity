import { useEffect, useMemo, useState } from "react";
import { Pencil, Plus, RefreshCw, ShieldCheck, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { RepositoryBranchView, RepositoryMergeRequestApprovalRuleView, UserView } from "@/pages/types";
import { extractErrorMessage } from "./issues-utils";
import { formatUserLabel } from "./repository-user-utils";
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
  const { mutateAsync: createRule, mutation: { isPending: isCreating } } = useCustomMutation<RawRecord>();
  const { mutateAsync: updateRule, mutation: { isPending: isUpdating } } = useCustomMutation<RawRecord>();
  const { mutateAsync: deleteRule, mutation: { isPending: isDeleting } } = useCustomMutation<RawRecord>();
  const [editingRuleID, setEditingRuleID] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [targetBranch, setTargetBranch] = useState(defaultBranch || "*");
  const [approvalsRequired, setApprovalsRequired] = useState("1");
  const [eligibleUserIDs, setEligibleUserIDs] = useState<string[]>([]);
  const [draftUserID, setDraftUserID] = useState("");
  const [codeOwner, setCodeOwner] = useState(false);
  const rules = useMemo(
    () => resolveApprovalRules(rulesQuery.result.data).map(normalizeApprovalRule),
    [rulesQuery.result.data],
  );
  const ruleStats = useMemo(
    () => ({
      total: rules.length,
      branchScoped: rules.filter((rule) => rule.target_branch && rule.target_branch !== "*").length,
      codeOwner: rules.filter((rule) => rule.code_owner).length,
    }),
    [rules],
  );
  const userByID = useMemo(() => new Map(users.map((user) => [user.id, user])), [users]);
  const availableUsers = users.filter((user) => !eligibleUserIDs.includes(user.id));
  const canAdminRules = permissions.mergeRequestMerge;
  const isBusy = isCreating || isUpdating || isDeleting;

  const reload = async () => {
    const result = await rulesQuery.query.refetch();
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
    if (rulesQuery.query.error) {
      onError(extractErrorMessage(rulesQuery.query.error));
    }
  }, [rulesQuery.query.error, onError]);

  return (
    <div className="flex flex-col gap-3 rounded-md border p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <p className="flex items-center gap-2 font-medium">
            <ShieldCheck className="size-4" />
            {t("Approval rules")}
          </p>
          <p className="text-xs text-muted-foreground">{t("Configure branch scoped approvals and CODEOWNERS requirements.")}</p>
        </div>
        <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
          <RefreshCw data-icon="inline-start" />
          {t("Reload")}
        </Button>
      </div>

      <div className="grid gap-2 md:grid-cols-3">
        <ApprovalRuleStat label={t("Rules")} value={ruleStats.total} />
        <ApprovalRuleStat label={t("Branch scoped")} value={ruleStats.branchScoped} />
        <ApprovalRuleStat label={t("CODEOWNERS")} value={ruleStats.codeOwner} />
      </div>

      {!canAdminRules ? (
        <Alert>
          <AlertDescription>{t("Your current project role can inspect approval rules, but cannot change them.")}</AlertDescription>
        </Alert>
      ) : null}

      <form className="flex flex-col gap-3 rounded-md border bg-muted/10 p-3" onSubmit={submitRule}>
        <div className="grid gap-3 lg:grid-cols-[1fr_180px_140px]">
          <div className="flex flex-col gap-1">
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
          <div className="flex flex-col gap-1">
            <Label className="text-xs text-muted-foreground">{t("Target branch")}</Label>
            <Select value={targetBranch || "*"} onValueChange={setTargetBranch} disabled={!canAdminRules || isBusy}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem value="*">{t("All branches")}</SelectItem>
                  {branches.map((branch) => (
                    <SelectItem key={branch.name} value={branch.name}>
                      {branch.name}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-1">
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
          <div className="flex flex-col gap-2">
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
                  <SelectGroup>
                    {availableUsers.map((user) => (
                      <SelectItem key={user.id} value={user.id}>
                        {formatUserLabel(user, user.id)}
                      </SelectItem>
                    ))}
                  </SelectGroup>
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
              <Plus data-icon="inline-start" />
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

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("Rule")}</TableHead>
              <TableHead>{t("Scope")}</TableHead>
              <TableHead>{t("Approvers")}</TableHead>
              <TableHead>{t("Requirement")}</TableHead>
              <TableHead className="text-right">{t("Actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rulesQuery.query.isFetching && !rulesQuery.query.data ? (
              <TableRow>
                <TableCell colSpan={5} className="text-muted-foreground">{t("Loading approval rules...")}</TableCell>
              </TableRow>
            ) : null}
            {!rulesQuery.query.isFetching && rules.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-muted-foreground">{t("No approval rules configured.")}</TableCell>
              </TableRow>
            ) : null}
            {rules.map((rule) => (
              <TableRow key={rule.id}>
                <TableCell>
                  <div className="flex flex-col gap-1">
                    <span className="font-medium">{rule.name}</span>
                    {rule.code_owner ? <Badge className="w-fit">{t("CODEOWNERS")}</Badge> : null}
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="outline">{rule.target_branch || "*"}</Badge>
                </TableCell>
                <TableCell className="max-w-[320px] text-muted-foreground">
                  {rule.eligible_user_ids.length > 0
                    ? rule.eligible_user_ids.map((userID) => formatUserLabel(userByID.get(userID), userID)).join(", ")
                    : t("Any approver can satisfy this rule.")}
                </TableCell>
                <TableCell>
                  <Badge variant="secondary">{rule.approvals_required} {t("approval(s)")}</Badge>
                </TableCell>
                <TableCell>
                  <div className="flex flex-wrap items-center justify-end gap-2">
                    <Button type="button" size="sm" variant="outline" disabled={!canAdminRules || isBusy} onClick={() => editRule(rule)}>
                      <Pencil data-icon="inline-start" />
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
                        <Trash2 data-icon="inline-start" />
                        {isDeleting ? t("Deleting...") : t("Delete")}
                      </Button>
                    </ConfirmAction>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};

const ApprovalRuleStat = ({ label, value }: { label: string; value: number }) => (
  <div className="rounded-md border bg-card p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
  </div>
);

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

