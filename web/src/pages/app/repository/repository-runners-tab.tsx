import { useEffect, useMemo, useState } from "react";
import { Activity, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import type { RepositoryRunnerView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import { RepositoryCIVariablesPanel } from "./repository-ci-variables-panel";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeBoolean, normalizeOptionalString, normalizeString, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryRunnersTabProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

type RawRunner = RawRecord;

export const RepositoryRunnersTab = ({ repoId, permissions, t, onError }: RepositoryRunnersTabProps): JSX.Element => {
  const runnersQuery = useCustom<RawRunner[]>({
    url: `/projects/${repoId}/runners`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: registerRunner, mutation: { isPending: isRegistering } } = useCustomMutation<RawRunner>();
  const { mutateAsync: deleteRunner, mutation: { isPending: isDeleting } } = useCustomMutation<RawRunner>();
  const runners = useMemo(
    () => resolveRunnerList(runnersQuery.result.data).map(normalizeRunner),
    [runnersQuery.result.data],
  );
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [tags, setTags] = useState("linux,go");
  const [lastToken, setLastToken] = useState<string | null>(null);
  const isLoadingRunners = runnersQuery.query.isFetching && !runnersQuery.query.data;
  const canAdminRunners = permissions.runnerAdmin;

  const loadRunners = async () => {
    const result = await runnersQuery.query.refetch();
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
    if (!runnersQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(runnersQuery.query.error));
  }, [runnersQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Runners")}</CardTitle>
        <CardDescription>{t("Register external runners and inspect project runner health.")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <RunnerStat label={t("Registered runners")} value={runners.length} description={runners.length > 0 ? t("Available to this project") : t("No capacity yet")} />
          <RunnerStat label={t("Online runners")} value={runners.filter((item) => item.status === "online").length} description={t("Heartbeat received recently")} active />
          <RunnerStat label={t("Offline runners")} value={runners.filter((item) => item.status !== "online").length} description={t("Needs runner attention")} />
          <RunnerStat label={t("Active runners")} value={runners.filter((item) => item.active).length} description={t("Eligible to claim jobs")} active />
        </div>

        {lastToken ? (
          <Alert>
            <AlertTitle>{t("Runner token generated")}</AlertTitle>
            <AlertDescription className="flex flex-col gap-2">
              <p>{t("Copy this token now. It is only shown after registration.")}</p>
              <code className="block overflow-auto rounded-md bg-background px-3 py-2 text-xs">{lastToken}</code>
            </AlertDescription>
          </Alert>
        ) : null}

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isComposerOpen ? "secondary" : "outline"}
            disabled={!canAdminRunners}
            onClick={() => setComposerOpen((current) => !current)}
          >
            <Plus data-icon="inline-start" />
            {isComposerOpen ? t("Hide runner form") : t("Register runner")}
          </Button>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={canAdminRunners ? "secondary" : "outline"}>
              {t("Role")}: {t(permissions.roleLabel)}
            </Badge>
            <Button type="button" size="sm" variant="ghost" onClick={() => void loadRunners()}>
              <RefreshCw data-icon="inline-start" />
              {t("Reload")}
            </Button>
          </div>
        </div>

        {!canAdminRunners ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect runners, but cannot register or delete them.")}</AlertDescription>
          </Alert>
        ) : null}

        {isComposerOpen ? (
          <form className="flex flex-col gap-3 rounded-md border bg-muted/10 p-3" onSubmit={submitRegisterRunner}>
            <div className="grid gap-3 md:grid-cols-[1fr_240px]">
              <div className="flex flex-col gap-1">
                <Label className="text-xs text-muted-foreground" htmlFor="runner-name">
                  {t("Runner name")}
                </Label>
                <Input
                  id="runner-name"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="linux-amd64"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label className="text-xs text-muted-foreground" htmlFor="runner-tags">
                  {t("Runner tags")}
                </Label>
                <Input
                  id="runner-tags"
                  value={tags}
                  onChange={(event) => setTags(event.target.value)}
                  placeholder="linux,go"
                />
              </div>
            </div>
            <div className="flex flex-col gap-1">
              <Label className="text-xs text-muted-foreground" htmlFor="runner-description">
                {t("Description")}
              </Label>
              <Input
                id="runner-description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder={t("Runner description optional")}
              />
            </div>
            <div className="flex justify-end">
              <Button type="submit" disabled={!canAdminRunners || isRegistering}>
                {isRegistering ? t("Registering runner...") : t("Register runner")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="flex flex-col gap-2 rounded-md border p-2">
          {isLoadingRunners ? <RunnerListSkeleton /> : null}
          {!isLoadingRunners && runners.length === 0 ? (
            <RunnerEmptyState
              canAdmin={canAdminRunners}
              t={t}
              onRegister={() => setComposerOpen(true)}
            />
          ) : null}
          {runners.map((runner) => (
            <div
              key={runner.id}
              className={cn(
                "flex flex-col gap-3 rounded-md border p-3",
                runner.status === "online" ? "border-primary/30 bg-primary/5" : "bg-background/60",
                !runner.active ? "opacity-80" : undefined,
              )}
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="flex min-w-0 flex-col gap-2">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="truncate font-medium">{runner.name}</p>
                    <RunnerStatusBadge runner={runner} t={t} />
                    <Badge variant={runner.active ? "secondary" : "outline"}>{runner.active ? t("active") : t("paused")}</Badge>
                  </div>
                  {runner.description ? <p className="text-sm text-muted-foreground">{runner.description}</p> : null}
                  <p className="text-xs text-muted-foreground">{getRunnerHealthDescription(runner, t)}</p>
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline">#{runner.id}</Badge>
                    {parseRunnerTags(runner.tags).map((tag) => (
                      <Badge key={tag} variant="outline">{tag}</Badge>
                    ))}
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
                    disabled={!canAdminRunners || isDeleting}
                  >
                    <Trash2 data-icon="inline-start" />
                    {t("Delete")}
                  </Button>
                </ConfirmAction>
              </div>
            </div>
          ))}
        </div>

        <Alert className="bg-muted/20">
          <AlertTitle>{t("Runner protocol")}</AlertTitle>
          <AlertDescription>
            {t("Use the token with /runners/heartbeat, /runners/jobs/claim, /runners/jobs/{id}/complete, and /runners/jobs/{id}/fail.")}
          </AlertDescription>
        </Alert>

        <RepositoryCIVariablesPanel repoId={repoId} permissions={permissions} t={t} onError={onError} />
      </CardContent>
    </Card>
  );
};

const RunnerStat = ({ label, value, description, active = false }: { label: string; value: number; description: string; active?: boolean }) => (
  <Card className={cn("shadow-none", active ? "border-primary/30 bg-primary/5" : "bg-muted/20")}>
    <CardContent className="flex flex-col gap-1 p-3">
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="text-2xl font-semibold">{value}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </CardContent>
  </Card>
);

const RunnerEmptyState = ({
  canAdmin,
  t,
  onRegister,
}: {
  canAdmin: boolean;
  t: (text: string) => string;
  onRegister: () => void;
}) => (
  <Alert className="border-dashed bg-muted/20">
    <AlertTitle>{t("No runners registered")}</AlertTitle>
    <AlertDescription className="flex flex-col gap-3">
      <p>{t("Register a project runner before pipelines can claim CI jobs on this repository.")}</p>
      <div className="flex flex-wrap gap-2">
        <Button type="button" size="sm" disabled={!canAdmin} onClick={onRegister}>
          <Plus data-icon="inline-start" />
          {t("Register first runner")}
        </Button>
        {!canAdmin ? <Badge variant="outline">{t("Runner admin required")}</Badge> : null}
      </div>
    </AlertDescription>
  </Alert>
);

const RunnerListSkeleton = () => (
  <div className="flex flex-col gap-2">
    <Skeleton className="h-24 w-full" />
    <Skeleton className="h-24 w-full" />
  </div>
);

const parseRunnerTags = (tags: string | null | undefined): string[] =>
  tags?.split(",").map((tag) => tag.trim()).filter(Boolean) ?? [];

const getRunnerHealthDescription = (runner: RepositoryRunnerView, t: (text: string) => string): string => {
  if (runner.status === "online" && runner.last_contact_at) {
    return `${t("Last heartbeat")}: ${formatRelativeTime(runner.last_contact_at)}`;
  }
  if (runner.status === "online") {
    return t("Online, but no heartbeat timestamp is available.");
  }
  return t("Offline runner. Check the runner process and token configuration.");
};

const RunnerStatusBadge = ({ runner, t }: { runner: RepositoryRunnerView; t: (text: string) => string }) => {
  const online = runner.status === "online";
  return (
    <Badge variant={online ? "default" : "secondary"} className="gap-1">
      <Activity className="size-3" />
      {online ? t("online") : t("offline")}
    </Badge>
  );
};

const resolveRunnerList = (payload: unknown): RawRunner[] => {
  if (Array.isArray(payload)) {
    return resolveRecordArray(payload);
  }
  return isRecord(payload) ? resolveRecordArray(payload.body ?? payload.Body) : [];
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

const normalizeStatus = (value: unknown): "online" | "offline" =>
  normalizeString(value).toLowerCase() === "online" ? "online" : "offline";
