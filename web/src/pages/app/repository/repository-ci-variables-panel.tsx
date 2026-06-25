import { useEffect, useMemo, useState } from "react";
import { KeyRound, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import type { RepositoryCIVariableView } from "@/pages/types";
import { extractErrorMessage } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeBoolean, normalizeOptionalString, normalizeString, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryCIVariablesPanelProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryCIVariablesPanel = ({ repoId, permissions, t, onError }: RepositoryCIVariablesPanelProps): JSX.Element => {
  const variablesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/ci/variables`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: upsertVariable, mutation: { isPending: isSaving } } = useCustomMutation<RawRecord>();
  const { mutateAsync: deleteVariable, mutation: { isPending: isDeleting } } = useCustomMutation<RawRecord>();
  const [key, setKey] = useState("");
  const [value, setValue] = useState("");
  const [masked, setMasked] = useState(true);
  const [protectedVariable, setProtectedVariable] = useState(false);
  const variables = useMemo(
    () => resolveVariables(variablesQuery.result.data).map(normalizeVariable),
    [variablesQuery.result.data],
  );
  const variableStats = useMemo(
    () => ({
      total: variables.length,
      masked: variables.filter((item) => item.masked).length,
      protected: variables.filter((item) => item.protected).length,
    }),
    [variables],
  );
  const canAdminVariables = permissions.runnerAdmin;
  const isBusy = isSaving || isDeleting;
  const isLoadingVariables = variablesQuery.query.isFetching && !variablesQuery.query.data;

  const reload = async () => {
    const result = await variablesQuery.query.refetch();
    onError(result.error ? extractErrorMessage(result.error) : null);
  };

  const submitVariable = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedKey = key.trim().toUpperCase();
    if (!normalizedKey) {
      onError(t("CI variable key is required."));
      return;
    }
    if (masked && value.trim().length < 8) {
      onError(t("Masked variable values must be at least 8 characters."));
      return;
    }
    onError(null);
    try {
      await upsertVariable({
        url: `/projects/${repoId}/ci/variables`,
        method: "patch",
        values: {
          key: normalizedKey,
          value,
          masked,
          protected: protectedVariable,
        },
      });
      setKey("");
      setValue("");
      setMasked(true);
      setProtectedVariable(false);
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitDeleteVariable = async (variable: RepositoryCIVariableView) => {
    onError(null);
    try {
      await deleteVariable({
        url: `/projects/${repoId}/ci/variables/${encodeURIComponent(variable.key)}`,
        method: "delete",
        values: {},
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    if (variablesQuery.query.error) {
      onError(extractErrorMessage(variablesQuery.query.error));
    }
  }, [variablesQuery.query.error, onError]);

  return (
    <div className="flex flex-col gap-4 rounded-md border p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="flex flex-col gap-2">
          <p className="flex items-center gap-2 font-medium">
            <KeyRound className="size-4" />
            {t("CI variables")}
          </p>
          <p className="text-xs text-muted-foreground">{t("Masked variables are injected into script jobs and redacted from logs.")}</p>
          <div className="flex flex-wrap gap-2">
            <Badge variant="secondary">{t("Variables")}: {variableStats.total}</Badge>
            <Badge variant="outline">{t("Masked")}: {variableStats.masked}</Badge>
            <Badge variant="outline">{t("Protected")}: {variableStats.protected}</Badge>
          </div>
        </div>
        <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
          <RefreshCw data-icon="inline-start" />
          {t("Reload")}
        </Button>
      </div>

      {!canAdminVariables ? (
        <Alert>
          <AlertDescription>{t("Your current project role can inspect runners, but cannot change CI variables.")}</AlertDescription>
        </Alert>
      ) : null}

      <form className="grid gap-3 rounded-md border bg-muted/10 p-3 lg:grid-cols-[180px_1fr_auto]" onSubmit={submitVariable}>
        <div className="flex flex-col gap-1">
          <Label className="text-xs text-muted-foreground" htmlFor="ci-variable-key">
            {t("Key")}
          </Label>
          <Input
            id="ci-variable-key"
            value={key}
            onChange={(event) => setKey(event.target.value.toUpperCase())}
            placeholder="DEPLOY_TOKEN"
            disabled={!canAdminVariables || isBusy}
          />
        </div>
        <div className="flex flex-col gap-1">
          <Label className="text-xs text-muted-foreground" htmlFor="ci-variable-value">
            {t("Value")}
          </Label>
          <Input
            id="ci-variable-value"
            value={value}
            onChange={(event) => setValue(event.target.value)}
            placeholder={t("Value is hidden after save")}
            disabled={!canAdminVariables || isBusy}
            type="password"
          />
        </div>
        <div className="flex flex-wrap items-end gap-2">
          <ToggleButton active={masked} disabled={!canAdminVariables || isBusy} label={t("Masked")} onToggle={() => setMasked((current) => !current)} />
          <ToggleButton active={protectedVariable} disabled={!canAdminVariables || isBusy} label={t("Protected")} onToggle={() => setProtectedVariable((current) => !current)} />
          <Button type="submit" disabled={!canAdminVariables || isBusy || !key.trim() || !value}>
            <Plus data-icon="inline-start" />
            {isSaving ? t("Saving...") : t("Save")}
          </Button>
        </div>
      </form>

      <div className="flex flex-col gap-2 rounded-md border p-2">
        {isLoadingVariables ? <VariableListSkeleton /> : null}
        {!isLoadingVariables && variables.length === 0 ? (
          <VariableEmptyState canAdmin={canAdminVariables} t={t} />
        ) : null}
        {variables.map((variable) => (
          <div
            key={variable.id || variable.key}
            className={cn(
              "flex flex-wrap items-center justify-between gap-3 rounded-md border p-3",
              variable.protected ? "border-primary/30 bg-primary/5" : "bg-background/60",
            )}
          >
            <div className="flex min-w-0 flex-col gap-2">
              <p className="font-mono text-sm font-medium">{variable.key}</p>
              <p className="text-xs text-muted-foreground">
                {variable.updated_at ? `${t("Updated")}: ${variable.updated_at}` : t("Value is hidden after save")}
              </p>
              <div className="flex flex-wrap gap-2">
                {variable.masked ? <Badge variant="secondary">{t("masked")}</Badge> : <Badge variant="outline">{t("visible")}</Badge>}
                {variable.protected ? <Badge variant="outline">{t("protected")}</Badge> : <Badge variant="outline">{t("unprotected")}</Badge>}
              </div>
            </div>
            <ConfirmAction
              title={t("Delete CI variable?")}
              description={t("Jobs started after deletion will no longer receive this variable.")}
              confirmLabel={t("Delete")}
              cancelLabel={t("Cancel")}
              onConfirm={() => void submitDeleteVariable(variable)}
            >
              <Button type="button" size="sm" variant="outline" disabled={!canAdminVariables || isBusy}>
                <Trash2 data-icon="inline-start" />
                {isDeleting ? t("Deleting...") : t("Delete")}
              </Button>
            </ConfirmAction>
          </div>
        ))}
      </div>
    </div>
  );
};

const VariableEmptyState = ({ canAdmin, t }: { canAdmin: boolean; t: (text: string) => string }) => (
  <Alert className="border-dashed bg-muted/20">
    <AlertTitle>{t("No CI variables configured")}</AlertTitle>
    <AlertDescription className="flex flex-col gap-3">
      <p>{t("Add masked variables for tokens, deployment keys, and environment values that jobs need at runtime.")}</p>
      <div className="flex flex-wrap gap-2">
        <Badge variant={canAdmin ? "secondary" : "outline"}>{canAdmin ? t("Use the form above") : t("Runner admin required")}</Badge>
      </div>
    </AlertDescription>
  </Alert>
);

const VariableListSkeleton = () => (
  <div className="flex flex-col gap-2">
    <Skeleton className="h-20 w-full" />
    <Skeleton className="h-20 w-full" />
  </div>
);

const ToggleButton = ({
  active,
  disabled,
  label,
  onToggle,
}: {
  active: boolean;
  disabled: boolean;
  label: string;
  onToggle: () => void;
}) => (
  <Button type="button" variant={active ? "secondary" : "outline"} disabled={disabled} aria-pressed={active} onClick={onToggle}>
    {label}
  </Button>
);

const resolveVariables = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  return isRecord(payload) ? resolveRecordArray(payload.body ?? payload.Body) : [];
};

const normalizeVariable = (raw: RawRecord): RepositoryCIVariableView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  key: normalizeString(raw.key ?? raw.Key),
  value: normalizeOptionalString(raw.value ?? raw.Value),
  masked: normalizeBoolean(raw.masked ?? raw.Masked),
  protected: normalizeBoolean(raw.protected ?? raw.Protected),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});
