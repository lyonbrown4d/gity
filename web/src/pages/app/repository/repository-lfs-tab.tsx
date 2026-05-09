import { useEffect, useMemo, useState } from "react";
import { Copy, Database, Lock, RefreshCw, Unlock } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { RepositoryLFSLockView, RepositoryLFSObjectView, RepositoryView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import { isRecord, normalizeNumber, normalizeOptionalString, normalizeString, resolveBody, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryLFSTabProps {
  repoId: string;
  repository: RepositoryView;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryLFSTab = ({ repoId, repository, t, onError }: RepositoryLFSTabProps): JSX.Element => {
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
  const { mutateAsync: createLock, isLoading: isCreatingLock } = useCustomMutation<RawRecord>();
  const { mutateAsync: unlockLock, isLoading: isUnlocking } = useCustomMutation<RawRecord>();
  const [lockPath, setLockPath] = useState("");
  const [isCopied, setCopied] = useState(false);
  const objects = useMemo(
    () => resolveObjectList(objectsQuery.data?.data).map(normalizeLFSObject),
    [objectsQuery.data?.data],
  );
  const locks = useMemo(
    () => resolveLockList(locksQuery.data?.data).map(normalizeLFSLock),
    [locksQuery.data?.data],
  );
  const totalBytes = objects.reduce((sum, item) => sum + item.byte_size, 0);
  const isLoadingObjects = objectsQuery.isFetching && !objectsQuery.data;
  const isLoadingLocks = locksQuery.isFetching && !locksQuery.data;
  const lfsEndpoint = `${repository.clone_http_url.replace(/\/$/, "")}/info/lfs`;
  const setupCommand = [
    "git lfs install",
    "git lfs track \"*.psd\"",
    "git add .gitattributes",
    "git commit -m \"Track large assets with Git LFS\"",
    "git push",
  ].join("\n");

  const reload = async () => {
    const [objectsResult, locksResult] = await Promise.all([objectsQuery.refetch(), locksQuery.refetch()]);
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
    const error = objectsQuery.error ?? locksQuery.error;
    if (!error) {
      return;
    }
    onError(extractErrorMessage(error));
  }, [objectsQuery.error, locksQuery.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Git LFS")}</CardTitle>
        <CardDescription>{t("Inspect stored LFS objects and manage project file locks.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <LFSStat label={t("Objects")} value={objects.length} />
          <LFSStat label={t("Storage")} value={formatBytes(totalBytes)} />
          <LFSStat label={t("Locks")} value={locks.length} />
        </div>

        <div className="grid gap-4 lg:grid-cols-[1fr_360px]">
          <div className="space-y-4">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="space-y-1">
                <p className="text-sm font-medium">{t("LFS endpoint")}</p>
                <p className="break-all text-xs text-muted-foreground">{lfsEndpoint}</p>
              </div>
              <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
                <RefreshCw className="size-4" />
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
                      <TableCell colSpan={3} className="text-sm text-muted-foreground">
                        {t("No LFS objects stored yet.")}
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
              <div className="space-y-3 p-3">
                <form className="grid gap-2 md:grid-cols-[1fr_auto]" onSubmit={submitCreateLock}>
                  <div className="space-y-1">
                    <Label htmlFor="lfs-lock-path">{t("Lock path")}</Label>
                    <Input
                      id="lfs-lock-path"
                      value={lockPath}
                      onChange={(event) => setLockPath(event.target.value)}
                      placeholder="assets/design.psd"
                    />
                  </div>
                  <div className="flex items-end">
                    <Button type="submit" disabled={isCreatingLock}>
                      <Lock className="size-4" />
                      {isCreatingLock ? t("Locking...") : t("Create lock")}
                    </Button>
                  </div>
                </form>

                <div className="space-y-2">
                  {isLoadingLocks ? <p className="text-sm text-muted-foreground">{t("Loading LFS locks...")}</p> : null}
                  {!isLoadingLocks && locks.length === 0 ? (
                    <p className="text-sm text-muted-foreground">{t("No file locks.")}</p>
                  ) : null}
                  {locks.map((item) => (
                    <div key={item.id} className="flex flex-wrap items-start justify-between gap-3 rounded-md border p-3">
                      <div className="min-w-0 space-y-1">
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
                        <Button type="button" size="sm" variant="outline" disabled={isUnlocking}>
                          <Unlock className="size-4" />
                          {t("Unlock")}
                        </Button>
                      </ConfirmAction>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          <div className="rounded-md border bg-muted/20 p-3">
            <div className="flex items-center justify-between gap-2">
              <p className="text-sm font-medium">{t("Client setup")}</p>
              <Button type="button" size="sm" variant="outline" onClick={() => void copySetupCommand()}>
                <Copy className="size-4" />
                {isCopied ? t("Copied") : t("Copy")}
              </Button>
            </div>
            <p className="mt-1 text-xs text-muted-foreground">
              {t("Use standard Git LFS clients against the project clone URL.")}
            </p>
            <ScrollArea className="mt-3 h-44 rounded-md border bg-background">
              <pre className="p-3 text-xs leading-6">{setupCommand}</pre>
            </ScrollArea>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const LFSStat = ({ label, value }: { label: string; value: number | string }) => (
  <div className="rounded-md border p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
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
