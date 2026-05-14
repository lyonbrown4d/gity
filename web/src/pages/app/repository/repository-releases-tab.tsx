import { useEffect, useMemo, useState } from "react";
import { ExternalLink, Link2, Plus, RefreshCw, Tag, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
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
  const { mutateAsync, isLoading } = useCustomMutation<RawRecord>();
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

  const releases = useMemo(
    () => resolveReleaseList(releasesQuery.data?.data).map(normalizeReleaseDetail),
    [releasesQuery.data?.data],
  );
  const tags = useMemo(
    () => resolveTagList(tagsQuery.data?.data).map(normalizeTag),
    [tagsQuery.data?.data],
  );
  const selectedRelease = useMemo(
    () => releases.find((item) => item.release.id === selectedReleaseId) ?? releases[0] ?? null,
    [releases, selectedReleaseId],
  );
  const linkCount = releases.reduce((total, item) => total + item.links.length, 0);
  const canAdminReleases = permissions.repositoryAdmin;
  const isLoadingReleases = releasesQuery.isFetching && !releasesQuery.data;
  const isLoadingTags = tagsQuery.isFetching && !tagsQuery.data;

  const reload = async () => {
    const [releaseResult, tagResult] = await Promise.all([releasesQuery.refetch(), tagsQuery.refetch()]);
    const error = releaseResult.error ?? tagResult.error;
    onError(error ? extractErrorMessage(error) : null);
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
    if (!releasesQuery.error && !tagsQuery.error) {
      return;
    }
    onError(extractErrorMessage(releasesQuery.error ?? tagsQuery.error));
  }, [onError, releasesQuery.error, tagsQuery.error]);

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
        <CardTitle>{t("Releases")}</CardTitle>
        <CardDescription>{t("Manage repository tags, release notes, and external release asset links.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <ReleaseStat label={t("Releases")} value={releases.length} />
          <ReleaseStat label={t("Tags")} value={tags.length} />
          <ReleaseStat label={t("Asset links")} value={linkCount} />
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap gap-2">
            <Button type="button" variant={isComposerOpen ? "secondary" : "outline"} disabled={!canAdminReleases} onClick={() => setComposerOpen((current) => !current)}>
              <Plus className="size-4" />
              {isComposerOpen ? t("Hide release form") : t("New release")}
            </Button>
            <Button type="button" variant={isLinkComposerOpen ? "secondary" : "outline"} disabled={!canAdminReleases || !selectedRelease} onClick={() => setLinkComposerOpen((current) => !current)}>
              <Link2 className="size-4" />
              {isLinkComposerOpen ? t("Hide link form") : t("Add asset link")}
            </Button>
          </div>
          <Button type="button" size="sm" variant="ghost" onClick={() => void reload()}>
            <RefreshCw className="size-4" />
            {t("Reload")}
          </Button>
        </div>

        {!canAdminReleases ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect releases, but cannot change them.")}</AlertDescription>
          </Alert>
        ) : null}

        {isComposerOpen ? (
          <form className="space-y-3 rounded-md border p-3" onSubmit={submitRelease}>
            <div className="grid gap-3 md:grid-cols-[1fr_1fr_160px]">
              <div className="space-y-1">
                <Label htmlFor="release-tag">{t("Tag name")}</Label>
                <Input id="release-tag" value={tagName} onChange={(event) => setTagName(event.target.value)} placeholder="v0.1.0" />
              </div>
              <div className="space-y-1">
                <Label htmlFor="release-source">{t("Source ref")}</Label>
                <Input id="release-source" value={sourceRef} onChange={(event) => setSourceRef(event.target.value)} placeholder={defaultBranch || "main"} />
              </div>
              <div className="space-y-1">
                <Label>{t("Create tag")}</Label>
                <Select value={createTag ? "yes" : "no"} onValueChange={(value) => setCreateTag(value === "yes")}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="yes">{t("Yes")}</SelectItem>
                    <SelectItem value="no">{t("No")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-1">
              <Label htmlFor="release-name">{t("Release name")}</Label>
              <Input id="release-name" value={releaseName} onChange={(event) => setReleaseName(event.target.value)} placeholder="Gity 0.1.0 beta" />
            </div>
            <div className="space-y-1">
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
            <div className="space-y-1">
              <Label htmlFor="release-link-name">{t("Link name")}</Label>
              <Input id="release-link-name" value={linkName} onChange={(event) => setLinkName(event.target.value)} placeholder="linux-amd64 tarball" />
            </div>
            <div className="space-y-1">
              <Label htmlFor="release-link-url">{t("URL")}</Label>
              <Input id="release-link-url" value={linkUrl} onChange={(event) => setLinkUrl(event.target.value)} placeholder="https://example.com/artifact.tar.gz" />
            </div>
            <div className="space-y-1">
              <Label>{t("Link type")}</Label>
              <Select value={linkType} onValueChange={setLinkType}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="package">package</SelectItem>
                  <SelectItem value="image">image</SelectItem>
                  <SelectItem value="runbook">runbook</SelectItem>
                  <SelectItem value="other">other</SelectItem>
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
          <div className="space-y-3">
            {isLoadingReleases ? <p className="text-sm text-muted-foreground">{t("Loading releases...")}</p> : null}
            {!isLoadingReleases && releases.length === 0 ? (
              <div className="rounded-md border p-4 text-sm text-muted-foreground">{t("No releases yet.")}</div>
            ) : null}
            {releases.map((item) => (
              <div
                key={item.release.id}
                role="button"
                tabIndex={0}
                className={`w-full rounded-md border p-4 text-left transition hover:bg-muted/40 ${selectedRelease?.release.id === item.release.id ? "border-primary bg-primary/5" : ""}`}
                onClick={() => setSelectedReleaseId(item.release.id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    setSelectedReleaseId(item.release.id);
                  }
                }}
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0 space-y-1">
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
                        <Trash2 className="size-4" />
                      </Button>
                    </ConfirmAction>
                  ) : null}
                </div>
                {item.links.length > 0 ? (
                  <div className="mt-3 flex flex-wrap gap-2">
                    {item.links.map((link) => (
                      <span key={link.id} className="inline-flex items-center gap-1 rounded-full border px-2 py-1 text-xs text-muted-foreground">
                        <Link2 className="size-3" />
                        {link.name}
                      </span>
                    ))}
                  </div>
                ) : null}
              </div>
            ))}
          </div>

          <div className="space-y-4">
            <div className="rounded-md border p-3">
              <div className="mb-3 flex items-center justify-between gap-2">
                <p className="font-medium">{t("Tags")}</p>
                {isLoadingTags ? <span className="text-xs text-muted-foreground">{t("Loading...")}</span> : null}
              </div>
              <div className="space-y-2">
                {tags.length === 0 && !isLoadingTags ? <p className="text-sm text-muted-foreground">{t("No tags yet.")}</p> : null}
                {tags.map((tag) => (
                  <div key={tag.name} className="rounded-md border p-2">
                    <div className="flex items-start justify-between gap-2">
                      <div className="min-w-0">
                        <p className="truncate text-sm font-medium">
                          <Tag className="mr-1 inline size-3.5" />
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
                            <Trash2 className="size-4" />
                          </Button>
                        </ConfirmAction>
                      ) : null}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="rounded-md border p-3">
              <p className="mb-3 font-medium">{t("Selected release assets")}</p>
              {!selectedRelease ? <p className="text-sm text-muted-foreground">{t("Select a release.")}</p> : null}
              {selectedRelease?.links.length === 0 ? <p className="text-sm text-muted-foreground">{t("No asset links.")}</p> : null}
              <div className="space-y-2">
                {selectedRelease?.links.map((link) => (
                  <div key={link.id} className="flex items-center justify-between gap-2 rounded-md border p-2">
                    <a className="min-w-0 text-sm underline underline-offset-4" href={link.url} target="_blank" rel="noreferrer">
                      <ExternalLink className="mr-1 inline size-3.5" />
                      {link.name}
                    </a>
                    <Badge variant="outline">{link.link_type}</Badge>
                    {canAdminReleases ? (
                      <Button type="button" size="icon" variant="ghost" onClick={() => void deleteLink(selectedRelease.release, link)}>
                        <Trash2 className="size-4" />
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

const ReleaseStat = ({ label, value }: { label: string; value: number }) => (
  <div className="rounded-md border p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
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
