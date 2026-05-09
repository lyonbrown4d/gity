import { useEffect, useMemo, useState } from "react";
import { Activity, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { RepositoryRunnerView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";

interface RepositoryRunnersTabProps {
  repoId: string;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

type RawRunner = Record<string, unknown>;

export const RepositoryRunnersTab = ({ repoId, t, onError }: RepositoryRunnersTabProps): JSX.Element => {
  const runnersQuery = useCustom<RawRunner[]>({
    url: `/projects/${repoId}/runners`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: registerRunner, isLoading: isRegistering } = useCustomMutation<RawRunner>();
  const { mutateAsync: deleteRunner, isLoading: isDeleting } = useCustomMutation<RawRunner>();
  const runners = useMemo(
    () => resolveRunnerList(runnersQuery.data?.data).map(normalizeRunner),
    [runnersQuery.data?.data],
  );
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [tags, setTags] = useState("linux,go");
  const [lastToken, setLastToken] = useState<string | null>(null);
  const isLoadingRunners = runnersQuery.isFetching && !runnersQuery.data;

  const loadRunners = async () => {
    const result = await runnersQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const submitRegisterRunner = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedName = name.trim();
    if (!normalizedName) {
      onError(t("Runner name is required"));
      return;
    }

    onError(null);
    try {
      const response = await registerRunner({
        url: `/projects/${repoId}/runners`,
        method: "post",
        values: {
          name: normalizedName,
          description: description.trim(),
          tags: tags.trim(),
        },
      });
      const payload = resolveRegistration(response.data);
      setLastToken(payload.token);
      setName("");
      setDescription("");
      setTags("linux,go");
      setComposerOpen(false);
      await loadRunners();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitDeleteRunner = async (runner: RepositoryRunnerView) => {
    onError(null);
    try {
      await deleteRunner({
        url: `/projects/${repoId}/runners/${runner.id}`,
        method: "delete",
        values: {},
      });
      await loadRunners();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    if (!repoId) {
      return;
    }
    onError(null);
  }, [repoId, onError]);

  useEffect(() => {
    if (!runnersQuery.error) {
      return;
    }
    onError(extractErrorMessage(runnersQuery.error));
  }, [runnersQuery.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Runners")}</CardTitle>
        <CardDescription>{t("Register external runners and inspect project runner health.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <RunnerStat label={t("Registered runners")} value={runners.length} />
          <RunnerStat label={t("Online runners")} value={runners.filter((item) => item.status === "online").length} />
          <RunnerStat label={t("Active runners")} value={runners.filter((item) => item.active).length} />
        </div>

        {lastToken ? (
          <div className="rounded-md border border-amber-500/30 bg-amber-500/5 p-3">
            <p className="text-sm font-medium">{t("Runner token")}</p>
            <p className="mt-1 text-xs text-muted-foreground">{t("Copy this token now. It is only shown after registration.")}</p>
            <code className="mt-2 block overflow-auto rounded-md bg-background px-3 py-2 text-xs">{lastToken}</code>
          </div>
        ) : null}

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isComposerOpen ? "secondary" : "outline"}
            onClick={() => setComposerOpen((current) => !current)}
          >
            <Plus className="size-4" />
            {isComposerOpen ? t("Hide runner form") : t("Register runner")}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadRunners()}>
            <RefreshCw className="size-4" />
            {t("Reload")}
          </Button>
        </div>

        {isComposerOpen ? (
          <form className="space-y-3 rounded-md border p-3" onSubmit={submitRegisterRunner}>
            <div className="grid gap-3 md:grid-cols-[1fr_240px]">
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="runner-name">
                  {t("Runner name")}
                </label>
                <Input
                  id="runner-name"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="linux-amd64"
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="runner-tags">
                  {t("Runner tags")}
                </label>
                <Input
                  id="runner-tags"
                  value={tags}
                  onChange={(event) => setTags(event.target.value)}
                  placeholder="linux,go"
                />
              </div>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium text-muted-foreground" htmlFor="runner-description">
                {t("Description")}
              </label>
              <Input
                id="runner-description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder={t("Runner description optional")}
              />
            </div>
            <div className="flex justify-end">
              <Button type="submit" disabled={isRegistering}>
                {isRegistering ? t("Registering runner...") : t("Register runner")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="space-y-2 rounded-md border p-2">
          {isLoadingRunners ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading runners...")}</p> : null}
          {!isLoadingRunners && runners.length === 0 ? (
            <p className="px-2 py-2 text-sm text-muted-foreground">{t("No runners registered.")}</p>
          ) : null}
          {runners.map((runner) => (
            <div key={runner.id} className="rounded-md border p-3">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="min-w-0 space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="font-medium">{runner.name}</p>
                    <RunnerStatusBadge runner={runner} t={t} />
                  </div>
                  {runner.description ? <p className="text-sm text-muted-foreground">{runner.description}</p> : null}
                  <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                    <span>#{runner.id}</span>
                    {runner.tags ? <span>{runner.tags}</span> : null}
                    {runner.last_contact_at ? <span>{formatRelativeTime(runner.last_contact_at)}</span> : <span>{t("No heartbeat yet")}</span>}
                  </div>
                </div>
                <ConfirmAction
                  title={t("Delete runner \"{name}\"?").replace("{name}", runner.name)}
                  description={t("This action cannot be undone.")}
                  confirmLabel={t("Delete")}
                  cancelLabel={t("Cancel")}
                  onConfirm={() => void submitDeleteRunner(runner)}
                >
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={isDeleting}
                  >
                    <Trash2 className="size-4" />
                    {t("Delete")}
                  </Button>
                </ConfirmAction>
              </div>
            </div>
          ))}
        </div>

        <div className="rounded-md border bg-muted/20 p-3">
          <p className="text-sm font-medium">{t("Runner protocol")}</p>
          <p className="mt-1 text-xs text-muted-foreground">
            {t("Use the token with /runners/heartbeat, /runners/jobs/claim, /runners/jobs/{id}/complete, and /runners/jobs/{id}/fail.")}
          </p>
        </div>
      </CardContent>
    </Card>
  );
};

const RunnerStat = ({ label, value }: { label: string; value: number }) => (
  <div className="rounded-md border p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
  </div>
);

const RunnerStatusBadge = ({ runner, t }: { runner: RepositoryRunnerView; t: (text: string) => string }) => {
  const online = runner.status === "online";
  return (
    <Badge variant={online ? "default" : "secondary"}>
      <Activity className="mr-1 size-3" />
      {online ? t("online") : t("offline")}
    </Badge>
  );
};

const resolveRunnerList = (payload: unknown): RawRunner[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  if (!isRecord(payload)) {
    return [];
  }
  const nested = payload.body ?? payload.Body;
  if (Array.isArray(nested)) {
    return nested.filter(isRecord);
  }
  return [];
};

const resolveRegistration = (payload: unknown): { token: string } => {
  const raw = isRecord(payload) && isRecord(payload.body ?? payload.Body)
    ? (payload.body ?? payload.Body)
    : payload;
  if (!isRecord(raw)) {
    return { token: "" };
  }
  return { token: normalizeString(raw.token ?? raw.Token) };
};

const normalizeRunner = (raw: RawRunner): RepositoryRunnerView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  name: normalizeString(raw.name ?? raw.Name) || "runner",
  description: normalizeOptionalString(raw.description ?? raw.Description),
  tags: normalizeOptionalString(raw.tags ?? raw.Tags),
  status: normalizeStatus(raw.status ?? raw.Status),
  active: normalizeBoolean(raw.active ?? raw.Active),
  last_contact_at: normalizeOptionalString(raw.last_contact_at ?? raw.LastContactAt),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const isRecord = (value: unknown): value is RawRunner =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const normalizeString = (value: unknown): string => {
  if (value === undefined || value === null) {
    return "";
  }
  return String(value);
};

const normalizeOptionalString = (value: unknown): string | null => {
  const normalized = normalizeString(value).trim();
  if (!normalized || normalized === "0001-01-01T00:00:00Z") {
    return null;
  }
  return normalized;
};

const normalizeBoolean = (value: unknown): boolean =>
  value === true || value === 1 || normalizeString(value) === "1" || normalizeString(value).toLowerCase() === "true";

const normalizeStatus = (value: unknown): "online" | "offline" =>
  normalizeString(value).toLowerCase() === "online" ? "online" : "offline";
