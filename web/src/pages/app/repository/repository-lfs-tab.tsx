import { useEffect, useMemo, useState } from "react";
import { Copy, Database, Lock, RefreshCw, Unlock } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { RepositoryLFSLockView, RepositoryLFSObjectView, RepositoryView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeNumber, normalizeOptionalString, normalizeString, resolveBody, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryLFSTabProps {
  repoId: string;
  repository: RepositoryView;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryLFSTab = ({ repoId, repository, permissions, t, onError }: RepositoryLFSTabProps): JSX.Element => {
  const objectsQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/lfs/objects`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const locksQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/lfs/locks`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: createLock, mutation: { isPending: isCreatingLock } } = useCustomMutation<RawRecord>();
  const { mutateAsync: unlockLock, mutation: { isPending: isUnlocking } } = useCustomMutation<RawRecord>();
  const [lockPath, setLockPath] = useState("");
  const [isCopied, setCopied] = useState(false);
  const objects = useMemo(
    () => resolveObjectList(objectsQuery.result.data).map(normalizeLFSObject),
    [objectsQuery.result.data],
  );
  const locks = useMemo(
    () => resolveLockList(locksQuery.result.data).map(normalizeLFSLock),
    [locksQuery.result.data],
  );
  const totalBytes = objects.reduce((sum, item) => sum + item.byte_size, 0);
  const isLoadingObjects = objectsQuery.query.isFetching && !objectsQuery.query.data;
  const isLoadingLocks = locksQuery.query.isFetching && !locksQuery.query.data;
  const canWriteLFS = permissions.canWrite;
  const lfsEndpoint = `${repository.clone_http_url.replace(/\/$/, "")}/info/lfs`;
  const setupCommand = [
    "git lfs install",
    "git lfs track \"*.psd\"",
    "git add .gitattributes",
    "git commit -m \"Track large assets with Git LFS\"",
    "git push",
  ].join("\n");

  const reload = async () => {
    const [objectsResult, locksResult] = await Promise.all([objectsQuery.query.refetch(), locksQuery.query.refetch()]);
    const error = objectsResult.error ?? locksResult.error;
    if (error) {
      onError(extractErrorMessage(error));
      return;
    }
    onError(null);
  };

  const submitCreateLock = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedPath = lockPath.trim();
    if (!normalizedPath) {
      onError(t("LFS lock path is required"));
      return;
    }

    onError(null);
    try {
      await createLock({
        url: `/projects/${repoId}/lfs/locks`,
        method: "post",
        values: { path: normalizedPath },
      });
      setLockPath("");
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitUnlock = async (lockItem: RepositoryLFSLockView) => {
    onError(null);
    try {
      await unlockLock({
        url: `/projects/${repoId}/lfs/locks/${lockItem.id}/unlock`,
        method: "post",
        values: { force: true },
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const copySetupCommand = async () => {
    try {
      await navigator.clipboard.writeText(setupCommand);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
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
    const error = objectsQuery.query.error ?? locksQuery.query.error;
    if (!error) {
      return;
    }
    onError(extractErrorMessage(error));
  }, [objectsQuery.query.error, locksQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="flex flex-col gap-1.5">
            <CardTitle>{t("Git LFS")}</CardTitle>
            <CardDescription>{t("Inspect stored LFS objects and manage project file locks.")}</CardDescription>
          </div>
          <Badge variant={canWriteLFS ? "secondary" : "outline"}>
            {canWriteLFS ? t("Locks enabled") : t("Read only")}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <LFSStat label={t("Objects")} value={objects.length} />
          <LFSStat label={t("Storage")} value={formatBytes(totalBytes)} />
          <LFSStat label={t("Locks")} value={locks.length} />
        </div>

        {!canWriteLFS ? (
          <Alert>
            <AlertTitle>{t("LFS locks are read-only")}</AlertTitle>
            <AlertDescription>{t("Your current project role can inspect LFS, but cannot manage locks.")}</AlertDescription>
          </Alert>
        ) : null}

        <div className="grid gap-4 lg:grid-cols-[1fr_360px]">
          <div className="flex flex-col gap-4">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex flex-col gap-1">
                <p className="text-sm font-medium">{t("LFS endpoint")}</p>
                <p className="break-all text-xs text-muted-foreground">{lfsEndpoint}</p>
              </div>
              <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
                <RefreshCw data-icon="inline-start" />
                {t("Reload")}
              </Button>
            </div>

            <div className="rounded-md border">
              <div className="flex items-center gap-2 border-b px-3 py-2">
                <Database className="size-4 text-muted-foreground" />
                <p className="font-medium">{t("Stored objects")}</p>
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("OID")}</TableHead>
                    <TableHead>{t("Size")}</TableHead>
                    <TableHead>{t("Updated")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoadingObjects ? (
                    <TableRow>
                      <TableCell colSpan={3} className="text-sm text-muted-foreground">
                        {t("Loading LFS objects...")}
                      </TableCell>
                    </TableRow>
                  ) : null}
                  {!isLoadingObjects && objects.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={3}>
                        <div className="flex flex-col gap-1 py-3 text-sm">
                          <span className="font-medium">{t("No LFS objects stored yet.")}</span>
                          <span className="text-muted-foreground">{t("Tracked large files will appear here after the first LFS push.")}</span>
                        </div>
                      </TableCell>
                    </TableRow>
                  ) : null}
                  {objects.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell className="max-w-[360px] truncate font-mono text-xs">{item.oid}</TableCell>
                      <TableCell>{formatBytes(item.byte_size)}</TableCell>
                      <TableCell>{item.updated_at ? formatRelativeTime(item.updated_at) : "--"}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            <div className="rounded-md border">
              <div className="flex items-center gap-2 border-b px-3 py-2">
                <Lock className="size-4 text-muted-foreground" />
                <p className="font-medium">{t("File locks")}</p>
              </div>
              <div className="flex flex-col gap-3 p-3">
                <form className="grid gap-2 md:grid-cols-[1fr_auto]" onSubmit={submitCreateLock}>
                  <div className="flex flex-col gap-1">
                    <Label htmlFor="lfs-lock-path">{t("Lock path")}</Label>
                    <Input
                      id="lfs-lock-path"
                      value={lockPath}
                      disabled={!canWriteLFS}
                      onChange={(event) => setLockPath(event.target.value)}
                      placeholder="assets/design.psd"
                    />
                  </div>
                  <div className="flex items-end">
                    <Button type="submit" disabled={!canWriteLFS || isCreatingLock}>
                      <Lock data-icon="inline-start" />
                      {isCreatingLock ? t("Locking...") : t("Create lock")}
                    </Button>
                  </div>
                </form>

                <div className="flex flex-col gap-2">
                  {isLoadingLocks ? <p className="text-sm text-muted-foreground">{t("Loading LFS locks...")}</p> : null}
                  {!isLoadingLocks && locks.length === 0 ? (
                    <LFSEmptyState
                      title={t("No file locks.")}
                      description={canWriteLFS ? t("Create a lock to reserve an LFS-managed path before editing.") : t("Active file locks will appear here.")}
                    />
                  ) : null}
                  {locks.map((item) => (
                    <div key={item.id} className="flex flex-wrap items-start justify-between gap-3 rounded-md border p-3">
                      <div className="flex min-w-0 flex-col gap-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="break-all font-medium">{item.path}</p>
                          <Badge variant="secondary">{item.owner.name || t("Unknown owner")}</Badge>
                        </div>
                        <p className="text-xs text-muted-foreground">
                          #{item.id}
                          {item.locked_at ? ` · ${formatRelativeTime(item.locked_at)}` : ""}
                        </p>
                      </div>
                      <ConfirmAction
                        title={t("Unlock \"{path}\"?").replace("{path}", item.path)}
                        description={t("This force-unlocks the file for the project.")}
                        confirmLabel={t("Unlock")}
                        cancelLabel={t("Cancel")}
                        onConfirm={() => void submitUnlock(item)}
                      >
                        <Button type="button" size="sm" variant="outline" disabled={!canWriteLFS || isUnlocking}>
                          <Unlock data-icon="inline-start" />
                          {t("Unlock")}
                        </Button>
                      </ConfirmAction>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          <div className="flex flex-col gap-3 rounded-md border bg-muted/20 p-3">
            <div className="flex items-center justify-between gap-2">
              <p className="text-sm font-medium">{t("Client setup")}</p>
              <Button type="button" size="sm" variant="outline" onClick={() => void copySetupCommand()}>
                <Copy data-icon="inline-start" />
                {isCopied ? t("Copied") : t("Copy")}
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              {t("Use standard Git LFS clients against the project clone URL.")}
            </p>
            <ScrollArea className="h-44 rounded-md border bg-background">
              <pre className="p-3 text-xs leading-6">{setupCommand}</pre>
            </ScrollArea>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const LFSStat = ({ label, value }: { label: string; value: number | string }) => (
  <div className="flex flex-col gap-1 rounded-md border p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
  </div>
);

const LFSEmptyState = ({ title, description }: { title: string; description: string }) => (
  <div className="flex flex-col items-center justify-center gap-2 rounded-md border border-dashed p-4 text-center">
    <Lock className="size-5 text-muted-foreground" />
    <div className="flex flex-col gap-1">
      <p className="text-sm font-medium">{title}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
  </div>
);

const resolveObjectList = (payload: unknown): RawRecord[] => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return [];
  }
  return resolveRecordArray(raw.objects ?? raw.Objects);
};

const resolveLockList = (payload: unknown): RawRecord[] => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return [];
  }
  return resolveRecordArray(raw.locks ?? raw.Locks);
};

const normalizeLFSObject = (raw: RawRecord): RepositoryLFSObjectView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  oid: normalizeString(raw.oid ?? raw.OID),
  byte_size: normalizeNumber(raw.byte_size ?? raw.ByteSize),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const normalizeLFSLock = (raw: RawRecord): RepositoryLFSLockView => {
  const owner = raw.owner ?? raw.Owner;
  return {
    id: normalizeString(raw.id ?? raw.ID),
    path: normalizeString(raw.path ?? raw.Path),
    locked_at: normalizeOptionalString(raw.locked_at ?? raw.LockedAt),
    owner: {
      name: isRecord(owner) ? normalizeString(owner.name ?? owner.Name) : "",
    },
  };
};

const formatBytes = (value: number): string => {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KiB`;
  }
  return `${(value / 1024 / 1024).toFixed(1)} MiB`;
};
