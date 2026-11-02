import { useEffect, useMemo, useState } from "react";
import { CheckCircle2, Copy, ExternalLink, Link2, Package, Plus, RefreshCw, Tag, Terminal, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { getApiBaseUrl } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { RepositoryReleaseDetailView, RepositoryReleaseLinkView, RepositoryReleaseView, RepositoryTagView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeBoolean, normalizeOptionalString, normalizeString, resolveBody, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryReleasesTabProps {
  repoId: string;
  defaultBranch: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryReleasesTab = ({ repoId, defaultBranch, permissions, t, onError }: RepositoryReleasesTabProps): JSX.Element => {
  const releasesQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/releases`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const tagsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/repository/tags`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync, mutation: { isPending: isLoading } } = useCustomMutation<RawRecord>();
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [isLinkComposerOpen, setLinkComposerOpen] = useState(false);
  const [selectedReleaseId, setSelectedReleaseId] = useState<string | null>(null);
  const [tagName, setTagName] = useState("");
  const [sourceRef, setSourceRef] = useState(defaultBranch || "main");
  const [releaseName, setReleaseName] = useState("");
  const [description, setDescription] = useState("");
  const [createTag, setCreateTag] = useState(true);
  const [linkName, setLinkName] = useState("");
  const [linkUrl, setLinkUrl] = useState("");
  const [linkType, setLinkType] = useState("package");
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);

  const releases = useMemo(
    () => resolveReleaseList(releasesQuery.result.data).map(normalizeReleaseDetail),
    [releasesQuery.result.data],
  );
  const tags = useMemo(
    () => resolveTagList(tagsQuery.result.data).map(normalizeTag),
    [tagsQuery.result.data],
  );
  const selectedRelease = useMemo(
    () => releases.find((item) => item.release.id === selectedReleaseId) ?? releases[0] ?? null,
    [releases, selectedReleaseId],
  );
  const linkCount = releases.reduce((total, item) => total + item.links.length, 0);
  const canAdminReleases = permissions.repositoryAdmin;
  const isLoadingReleases = releasesQuery.query.isFetching && !releasesQuery.query.data;
  const isLoadingTags = tagsQuery.query.isFetching && !tagsQuery.query.data;
  const releaseGuide = useMemo(() => buildReleaseGuide(repoId, defaultBranch, selectedRelease), [repoId, defaultBranch, selectedRelease]);

  const reload = async () => {
    const [releaseResult, tagResult] = await Promise.all([releasesQuery.query.refetch(), tagsQuery.query.refetch()]);
    const error = releaseResult.error ?? tagResult.error;
    onError(error ? extractErrorMessage(error) : null);
  };

  const copyCommand = async (command: string) => {
    try {
      await navigator.clipboard.writeText(command);
      setCopiedCommand(command);
      window.setTimeout(() => setCopiedCommand((current) => (current === command ? null : current)), 1400);
      onError(null);
    } catch {
      onError(t("Failed to copy command."));
    }
  };

  const submitRelease = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedTag = tagName.trim();
    if (!normalizedTag) {
      onError(t("Tag name is required."));
      return;
    }
    try {
      onError(null);
      await mutateAsync({
        url: `/projects/${repoId}/releases`,
        method: "post",
        values: {
          tag_name: normalizedTag,
          name: releaseName.trim() || normalizedTag,
          description: description.trim(),
          source_ref: sourceRef.trim() || defaultBranch || "main",
          create_tag: createTag,
        },
      });
      setComposerOpen(false);
      setTagName("");
      setReleaseName("");
      setDescription("");
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitLink = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedRelease) {
      onError(t("Select a release first."));
      return;
    }
    if (!linkName.trim() || !linkUrl.trim()) {
      onError(t("Release link name and URL are required."));
      return;
    }
    try {
      onError(null);
      await mutateAsync({
        url: `/projects/${repoId}/releases/${selectedRelease.release.id}/links`,
        method: "post",
        values: {
          name: linkName.trim(),
          url: linkUrl.trim(),
          link_type: linkType,
        },
      });
      setLinkComposerOpen(false);
      setLinkName("");
      setLinkUrl("");
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const deleteRelease = async (release: RepositoryReleaseView) => {
    try {
      onError(null);
      await mutateAsync({
        url: `/projects/${repoId}/releases/${release.id}`,
        method: "delete",
        values: {},
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const deleteLink = async (release: RepositoryReleaseView, link: RepositoryReleaseLinkView) => {
    try {
      onError(null);
      await mutateAsync({
        url: `/projects/${repoId}/releases/${release.id}/links/${link.id}`,
        method: "delete",
        values: {},
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const deleteTag = async (tag: RepositoryTagView) => {
    try {
      onError(null);
      await mutateAsync({
        url: `/projects/${repoId}/repository/tags?name=${encodeURIComponent(tag.name)}`,
        method: "delete",
        values: {},
      });
      await reload();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    if (!releasesQuery.query.error && !tagsQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(releasesQuery.query.error ?? tagsQuery.query.error));
  }, [onError, releasesQuery.query.error, tagsQuery.query.error]);

  useEffect(() => {
    if (selectedReleaseId && releases.some((item) => item.release.id === selectedReleaseId)) {
      return;
    }
    setSelectedReleaseId(releases[0]?.release.id ?? null);
  }, [releases, selectedReleaseId]);

  useEffect(() => {
    setSourceRef(defaultBranch || "main");
  }, [defaultBranch]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="flex flex-col gap-1.5">
            <CardTitle>{t("Releases")}</CardTitle>
            <CardDescription>{t("Manage repository tags, release notes, and external release asset links.")}</CardDescription>
          </div>
          <Badge variant={canAdminReleases ? "secondary" : "outline"}>
            {canAdminReleases ? t("Admin actions enabled") : t("Read only")}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <ReleaseStat label={t("Releases")} value={releases.length} />
          <ReleaseStat label={t("Tags")} value={tags.length} />
          <ReleaseStat label={t("Asset links")} value={linkCount} />
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap gap-2">
            <Button type="button" variant={isComposerOpen ? "secondary" : "default"} disabled={!canAdminReleases} onClick={() => setComposerOpen((current) => !current)}>
              <Plus data-icon="inline-start" />
              {isComposerOpen ? t("Hide release form") : t("New release")}
            </Button>
            <Button type="button" variant={isLinkComposerOpen ? "secondary" : "outline"} disabled={!canAdminReleases || !selectedRelease} onClick={() => setLinkComposerOpen((current) => !current)}>
              <Link2 data-icon="inline-start" />
              {isLinkComposerOpen ? t("Hide link form") : t("Add asset link")}
            </Button>
          </div>
          <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
            <RefreshCw data-icon="inline-start" />
            {t("Reload")}
          </Button>
        </div>

        {!canAdminReleases ? (
          <Alert>
            <AlertTitle>{t("Release management is read-only")}</AlertTitle>
            <AlertDescription>{t("Your current project role can inspect releases, but cannot change them.")}</AlertDescription>
          </Alert>
        ) : null}

        {isComposerOpen ? (
          <form className="flex flex-col gap-3 rounded-md border p-3" onSubmit={submitRelease}>
            <div className="grid gap-3 md:grid-cols-[1fr_1fr_160px]">
              <div className="flex flex-col gap-1">
                <Label htmlFor="release-tag">{t("Tag name")}</Label>
                <Input id="release-tag" value={tagName} onChange={(event) => setTagName(event.target.value)} placeholder="v0.1.0" />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="release-source">{t("Source ref")}</Label>
                <Input id="release-source" value={sourceRef} onChange={(event) => setSourceRef(event.target.value)} placeholder={defaultBranch || "main"} />
              </div>
              <div className="flex flex-col gap-1">
                <Label>{t("Create tag")}</Label>
                <Select value={createTag ? "yes" : "no"} onValueChange={(value) => setCreateTag(value === "yes")}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="yes">{t("Yes")}</SelectItem>
                      <SelectItem value="no">{t("No")}</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="release-name">{t("Release name")}</Label>
              <Input id="release-name" value={releaseName} onChange={(event) => setReleaseName(event.target.value)} placeholder="Gity 0.1.0 beta" />
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="release-description">{t("Release notes")}</Label>
              <Textarea id="release-description" className="min-h-28" value={description} onChange={(event) => setDescription(event.target.value)} placeholder={t("Describe user-facing changes, upgrade notes, and known limitations.")} />
            </div>
            <div className="flex justify-end">
              <Button type="submit" disabled={!canAdminReleases || isLoading}>
                {isLoading ? t("Saving release...") : t("Create release")}
              </Button>
            </div>
          </form>
        ) : null}

        {isLinkComposerOpen && selectedRelease ? (
          <form className="grid gap-3 rounded-md border p-3 md:grid-cols-[1fr_1.6fr_160px_auto]" onSubmit={submitLink}>
            <div className="flex flex-col gap-1">
              <Label htmlFor="release-link-name">{t("Link name")}</Label>
              <Input id="release-link-name" value={linkName} onChange={(event) => setLinkName(event.target.value)} placeholder="linux-amd64 tarball" />
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="release-link-url">{t("URL")}</Label>
              <Input id="release-link-url" value={linkUrl} onChange={(event) => setLinkUrl(event.target.value)} placeholder="https://example.com/artifact.tar.gz" />
            </div>
            <div className="flex flex-col gap-1">
              <Label>{t("Link type")}</Label>
              <Select value={linkType} onValueChange={setLinkType}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectItem value="package">package</SelectItem>
                    <SelectItem value="image">image</SelectItem>
                    <SelectItem value="runbook">runbook</SelectItem>
                    <SelectItem value="other">other</SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="flex items-end">
              <Button type="submit" disabled={!canAdminReleases || isLoading}>
                {t("Add link")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="grid gap-4 xl:grid-cols-[1fr_320px]">
          <div className="flex flex-col gap-3">
            {isLoadingReleases ? <p className="text-sm text-muted-foreground">{t("Loading releases...")}</p> : null}
            {!isLoadingReleases && releases.length === 0 ? (
              <ReleaseEmptyState
                title={t("No releases yet.")}
                description={canAdminReleases ? t("Create a release from an existing tag or source ref.") : t("Releases will appear here after maintainers publish them.")}
              />
            ) : null}
            {releases.map((item) => (
              <div
                key={item.release.id}
                role="button"
                tabIndex={0}
                className={cn(
                  "w-full rounded-md border p-4 text-left transition hover:bg-muted/40",
                  selectedRelease?.release.id === item.release.id ? "border-primary bg-primary/5" : "",
                )}
                onClick={() => setSelectedReleaseId(item.release.id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    setSelectedReleaseId(item.release.id);
                  }
                }}
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="flex min-w-0 flex-col gap-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <h3 className="text-lg font-semibold">{item.release.name}</h3>
                      <Badge variant="secondary">{item.release.tag_name}</Badge>
                      {item.tag ? <Badge variant="outline">{shortSHA(item.tag.target_sha)}</Badge> : <Badge variant="outline" className="border-destructive text-destructive">{t("Missing tag")}</Badge>}
                    </div>
                    <p className="whitespace-pre-wrap text-sm text-muted-foreground">{item.release.description || t("No release notes.")}</p>
                    <p className="text-xs text-muted-foreground">
                      {item.release.released_at ? formatRelativeTime(item.release.released_at) : t("No release time")}
                    </p>
                  </div>
                  {canAdminReleases ? (
                    <ConfirmAction
                      title={t("Delete release \"{name}\"?").replace("{name}", item.release.name)}
                      description={t("This removes release metadata but keeps the Git tag.")}
                      confirmLabel={t("Delete")}
                      cancelLabel={t("Cancel")}
                      onConfirm={() => void deleteRelease(item.release)}
                    >
                      <Button type="button" size="sm" variant="ghost" onClick={(event) => event.stopPropagation()}>
                        <Trash2 />
                      </Button>
                    </ConfirmAction>
                  ) : null}
                </div>
                {item.links.length > 0 ? (
                  <div className="flex flex-wrap gap-2 pt-3">
                    {item.links.map((link) => (
                      <Badge key={link.id} variant="outline" className="gap-1 font-normal text-muted-foreground">
                        <Link2 className="size-3" />
                        {link.name}
                      </Badge>
                    ))}
                  </div>
                ) : null}
              </div>
            ))}
          </div>

          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-3 rounded-md border p-3">
              <div className="flex items-center justify-between gap-2">
                <p className="font-medium">{t("Tags")}</p>
                {isLoadingTags ? <span className="text-xs text-muted-foreground">{t("Loading...")}</span> : null}
              </div>
              <div className="flex flex-col gap-2">
                {tags.length === 0 && !isLoadingTags ? (
                  <ReleaseEmptyState title={t("No tags yet.")} description={t("Tags created for releases will appear here.")} />
                ) : null}
                {tags.map((tag) => (
                  <div key={tag.name} className="rounded-md border p-2">
                    <div className="flex items-start justify-between gap-2">
                      <div className="min-w-0">
                        <p className="flex items-center gap-1 truncate text-sm font-medium">
                          <Tag className="size-3.5" />
                          {tag.name}
                        </p>
                        <p className="font-mono text-xs text-muted-foreground">{shortSHA(tag.target_sha)}</p>
                      </div>
                      {canAdminReleases ? (
                        <ConfirmAction
                          title={t("Delete tag \"{name}\"?").replace("{name}", tag.name)}
                          description={t("Deleting a Git tag can affect release consumers.")}
                          confirmLabel={t("Delete")}
                          cancelLabel={t("Cancel")}
                          onConfirm={() => void deleteTag(tag)}
                        >
                          <Button type="button" size="icon" variant="ghost">
                            <Trash2 />
                          </Button>
                        </ConfirmAction>
                      ) : null}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="flex flex-col gap-3 rounded-md border p-3">
              <p className="font-medium">{t("Selected release assets")}</p>
              {!selectedRelease ? (
                <ReleaseEmptyState title={t("Select a release.")} description={t("Release asset links are shown for the selected release.")} />
              ) : null}
              {selectedRelease?.links.length === 0 ? (
                <ReleaseEmptyState title={t("No asset links.")} description={canAdminReleases ? t("Add an asset link for downloadable artifacts or runbooks.") : t("Asset links will appear here after they are added.")} />
              ) : null}
              <div className="flex flex-col gap-2">
                {selectedRelease?.links.map((link) => (
                  <div key={link.id} className="flex items-center justify-between gap-2 rounded-md border p-2">
                    <a className="inline-flex min-w-0 items-center gap-1 text-sm underline underline-offset-4" href={link.url} target="_blank" rel="noreferrer">
                      <ExternalLink className="size-3.5" />
                      {link.name}
                    </a>
                    <Badge variant="outline">{link.link_type}</Badge>
                    {canAdminReleases ? (
                      <Button type="button" size="icon" variant="ghost" onClick={() => void deleteLink(selectedRelease.release, link)}>
                        <Trash2 />
                      </Button>
                    ) : null}
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

interface ReleaseGuide {
  checklist: Array<{ label: string; done: boolean }>;
  commands: Array<{ label: string; value: string }>;
  packageHint: string;
}

const ReleaseReadinessGuide = ({
  guide,
  copiedCommand,
  t,
  onCopy,
}: {
  guide: ReleaseGuide;
  copiedCommand: string | null;
  t: (text: string) => string;
  onCopy: (command: string) => void;
}) => (
  <div className="grid gap-3 rounded-xl border border-border/80 bg-muted/20 p-3 xl:grid-cols-[280px_1fr]">
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <Terminal className="size-4 text-primary" />
        <p className="font-medium">{t("Release readiness")}</p>
      </div>
      <p className="text-xs leading-5 text-muted-foreground">
        {t("Use this checklist to connect Git tags, package artifacts, and release links into one consumable delivery flow.")}
      </p>
      <div className="flex flex-col gap-2">
        {guide.checklist.map((item) => (
          <div key={item.label} className="flex items-center gap-2 text-sm">
            <CheckCircle2 className={cn("size-4", item.done ? "text-emerald-500" : "text-muted-foreground")} />
            <span className={item.done ? "text-foreground" : "text-muted-foreground"}>{t(item.label)}</span>
          </div>
        ))}
      </div>
      <div className="rounded-lg border bg-background/80 p-3 text-xs leading-5 text-muted-foreground">
        <div className="mb-1 flex items-center gap-2 font-medium text-foreground">
          <Package className="size-3.5" />
          {t("Package handoff")}
        </div>
        {t(guide.packageHint)}
      </div>
    </div>
    <div className="grid gap-2 md:grid-cols-2">
      {guide.commands.map((command) => (
        <ReleaseCommandLine
          key={`${command.label}:${command.value}`}
          label={t(command.label)}
          value={command.value}
          copiedCommand={copiedCommand}
          t={t}
          onCopy={onCopy}
        />
      ))}
    </div>
  </div>
);

const ReleaseCommandLine = ({
  label,
  value,
  copiedCommand,
  t,
  onCopy,
}: {
  label: string;
  value: string;
  copiedCommand: string | null;
  t: (text: string) => string;
  onCopy: (command: string) => void;
}) => (
  <div className="flex flex-col gap-1 rounded-lg border bg-background/80 p-3">
    <div className="flex items-center justify-between gap-2">
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
      <Button type="button" size="sm" variant="ghost" className="h-7 px-2" onClick={() => onCopy(value)}>
        <Copy className="size-3.5" />
        {copiedCommand === value ? t("Copied") : t("Copy")}
      </Button>
    </div>
    <code className="block overflow-x-auto rounded-md bg-muted px-3 py-2 font-mono text-xs leading-5 text-foreground">
      {value}
    </code>
  </div>
);
const ReleaseStat = ({ label, value }: { label: string; value: number }) => (
  <div className="flex flex-col gap-1 rounded-md border p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
  </div>
);

const ReleaseEmptyState = ({ title, description }: { title: string; description: string }) => (
  <div className="flex flex-col items-center justify-center gap-2 rounded-md border border-dashed p-4 text-center">
    <Tag className="size-5 text-muted-foreground" />
    <div className="flex flex-col gap-1">
      <p className="text-sm font-medium">{title}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
  </div>
);

const resolveReleaseList = (payload: unknown): RawRecord[] => resolveRecordArray(resolveBody(payload));

const resolveTagList = (payload: unknown): RawRecord[] => resolveRecordArray(resolveBody(payload));

const normalizeReleaseDetail = (value: unknown): RepositoryReleaseDetailView => {
  const raw = isRecord(value) ? value : {};
  return {
    release: normalizeRelease(raw.release ?? raw.Release),
    links: resolveRecordArray(raw.links ?? raw.Links).map(normalizeReleaseLink),
    tag: isRecord(raw.tag ?? raw.Tag) ? normalizeTag(raw.tag ?? raw.Tag) : null,
  };
};

const normalizeRelease = (value: unknown): RepositoryReleaseView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    tag_name: normalizeString(raw.tag_name ?? raw.TagName),
    name: normalizeString(raw.name ?? raw.Name),
    description: normalizeOptionalString(raw.description ?? raw.Description),
    created_by_user_id: normalizeString(raw.created_by_user_id ?? raw.CreatedByUserID),
    released_at: normalizeOptionalString(raw.released_at ?? raw.ReleasedAt),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const normalizeReleaseLink = (value: unknown): RepositoryReleaseLinkView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_release_id: normalizeString(raw.project_release_id ?? raw.ProjectReleaseID),
    name: normalizeString(raw.name ?? raw.Name),
    url: normalizeString(raw.url ?? raw.URL),
    link_type: normalizeString(raw.link_type ?? raw.LinkType) || "other",
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const normalizeTag = (value: unknown): RepositoryTagView => {
  const raw = isRecord(value) ? value : {};
  return {
    name: normalizeString(raw.name ?? raw.Name),
    target_sha: normalizeString(raw.target_sha ?? raw.TargetSHA),
    message: normalizeOptionalString(raw.message ?? raw.Message),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    annotated: normalizeBoolean(raw.annotated ?? raw.Annotated),
    object_sha: normalizeString(raw.object_sha ?? raw.ObjectSHA),
    object_type: normalizeString(raw.object_type ?? raw.ObjectType),
  };
};

const shortSHA = (value: string): string => (value ? value.slice(0, 8) : "--------");

const buildReleaseGuide = (repoId: string, defaultBranch: string, selectedRelease: RepositoryReleaseDetailView | null): ReleaseGuide => {
  const tagName = selectedRelease?.release.tag_name || "v0.1.0";
  const releaseID = selectedRelease?.release.id || "<release-id>";
  const packageEndpoint = absoluteApiUrl(`/projects/${repoId}/packages`);
  const releaseEndpoint = absoluteApiUrl(`/projects/${repoId}/releases`);
  const assetEndpoint = `${releaseEndpoint}/${releaseID}/links`;
  const branch = defaultBranch || "main";
  return {
    checklist: [
      { label: "Git tag exists", done: Boolean(selectedRelease?.tag) },
      { label: "Release notes are written", done: Boolean(selectedRelease?.release.description) },
      { label: "At least one asset link is attached", done: Boolean(selectedRelease?.links.length) },
      { label: "Package artifact is published", done: true },
    ],
    packageHint: "Publish the build output to the package registry first, then add the package download URL or external artifact URL as a release asset link.",
    commands: [
      { label: "Create local tag", value: `git tag -a ${tagName} ${branch} -m "${tagName}"` },
      { label: "Push tag", value: `git push origin ${tagName}` },
      { label: "Publish package artifact", value: `curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./artifact.tar.gz ${packageEndpoint}/generic/my-package/${tagName}/artifact.tar.gz` },
      { label: "Attach release asset", value: `curl --request POST --header "Authorization: Bearer <token>" --header "Content-Type: application/json" --data '{"name":"artifact.tar.gz","url":"${packageEndpoint}/generic/my-package/${tagName}/artifact.tar.gz","link_type":"package"}' ${assetEndpoint}` },
    ],
  };
};

const absoluteApiUrl = (path: string): string => {
  const base = getApiBaseUrl().replace(/\/+$/, "");
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  if (/^https?:\/\//i.test(base)) {
    return `${base}${normalizedPath}`;
  }
  if (typeof window === "undefined") {
    return `${base}${normalizedPath}`;
  }
  return `${window.location.origin}${base}${normalizedPath}`;
};
