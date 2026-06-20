import { useEffect, useMemo, useState } from "react";
import { BookOpen, Edit3, Eye, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useCustom, useCustomMutation, useGetIdentity } from "@refinedev/core";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import type { RepositoryWikiPageView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeOptionalString, normalizeString, resolveRecordArray, type RawRecord } from "./repository-normalizers";
import { renderMarkdown } from "./repository-utils";

interface RepositoryWikiTabProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

type RawWikiPage = RawRecord;

export const RepositoryWikiTab = ({ repoId, permissions, t, onError }: RepositoryWikiTabProps): JSX.Element => {
  const identityQuery = useGetIdentity<{ id?: string | number }>({});
  const pagesQuery = useCustom<RawWikiPage[]>({
    url: `/projects/${repoId}/wiki/pages`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: createWikiPage, mutation: { isPending: isCreatingPage } } = useCustomMutation<RawWikiPage>();
  const { mutateAsync: updateWikiPage, mutation: { isPending: isUpdatingPage } } = useCustomMutation<RawWikiPage>();
  const { mutateAsync: deleteWikiPage, mutation: { isPending: isDeletingPage } } = useCustomMutation<RawWikiPage>();

  const currentUserId = normalizeString(identityQuery.data?.id);
  const pages = useMemo(
    () => resolveWikiList(pagesQuery.result.data).map(normalizeWikiPage),
    [pagesQuery.result.data],
  );
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
  const selectedPage = useMemo(
    () => pages.find((item) => item.slug === selectedSlug) ?? null,
    [pages, selectedSlug],
  );
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newSlug, setNewSlug] = useState("");
  const [newContent, setNewContent] = useState("");
  const [authorUserId, setAuthorUserId] = useState("");
  const [draftTitle, setDraftTitle] = useState("");
  const [draftContent, setDraftContent] = useState("");
  const [editorUserId, setEditorUserId] = useState("");
  const [previewHtml, setPreviewHtml] = useState("");
  const isLoadingPages = pagesQuery.query.isFetching && !pagesQuery.query.data;
  const canWriteWiki = permissions.wikiWrite;

  const loadPages = async () => {
    const result = await pagesQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const submitCreatePage = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const title = newTitle.trim();
    const authorID = parseRequiredUserID(authorUserId);
    if (!title) {
      onError(t("Wiki page title is required"));
      return;
    }
    if (authorID === null) {
      onError(t("Author user ID is required"));
      return;
    }

    onError(null);
    try {
      const response = await createWikiPage({
        url: `/projects/${repoId}/wiki/pages`,
        method: "post",
        values: {
          title,
          slug: newSlug.trim() || undefined,
          content: newContent,
          format: "markdown",
          author_user_id: authorID,
        },
      });
      const created = normalizeWikiPage(resolveWikiPage(response.data));
      setNewTitle("");
      setNewSlug("");
      setNewContent("");
      setComposerOpen(false);
      setSelectedSlug(created.slug || null);
      await loadPages();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitUpdatePage = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedPage) {
      return;
    }
    const title = draftTitle.trim();
    const editorID = parseOptionalUserID(editorUserId);
    if (!title) {
      onError(t("Wiki page title is required"));
      return;
    }
    if (editorID === null) {
      onError(t("Editor user ID must be a positive number"));
      return;
    }

    onError(null);
    try {
      await updateWikiPage({
        url: `/projects/${repoId}/wiki/pages/${encodeURIComponent(selectedPage.slug)}`,
        method: "patch",
        values: {
          title,
          content: draftContent,
          editor_user_id: editorID,
        },
      });
      await loadPages();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitDeletePage = async () => {
    if (!selectedPage) {
      return;
    }
    onError(null);
    try {
      await deleteWikiPage({
        url: `/projects/${repoId}/wiki/pages/${encodeURIComponent(selectedPage.slug)}`,
        method: "delete",
        values: {},
      });
      setSelectedSlug(null);
      await loadPages();
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
    if (!pagesQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(pagesQuery.query.error));
  }, [pagesQuery.query.error, onError]);

  useEffect(() => {
    if (!currentUserId) {
      return;
    }
    if (!authorUserId) {
      setAuthorUserId(currentUserId);
    }
    if (!editorUserId) {
      setEditorUserId(currentUserId);
    }
  }, [currentUserId, authorUserId, editorUserId]);

  useEffect(() => {
    if (pages.length === 0) {
      setSelectedSlug(null);
      return;
    }
    if (!selectedSlug || !pages.some((item) => item.slug === selectedSlug)) {
      setSelectedSlug(pages[0].slug);
    }
  }, [pages, selectedSlug]);

  useEffect(() => {
    if (!selectedPage) {
      setDraftTitle("");
      setDraftContent("");
      return;
    }
    setDraftTitle(selectedPage.title);
    setDraftContent(selectedPage.content);
    setEditorUserId(selectedPage.last_edited_by_user_id || selectedPage.author_user_id);
  }, [selectedPage]);

  useEffect(() => {
    let active = true;
    if (!draftContent.trim()) {
      setPreviewHtml("<p>Empty wiki page.</p>");
      return () => {
        active = false;
      };
    }
    renderMarkdown(draftContent)
      .then((html) => {
        if (active) {
          setPreviewHtml(html);
        }
      })
      .catch(() => {
        if (active) {
          setPreviewHtml("<p>Failed to render markdown preview.</p>");
        }
      });
    return () => {
      active = false;
    };
  }, [draftContent]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Wiki")}</CardTitle>
        <CardDescription>{t("Maintain project wiki pages with markdown content.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <div className="rounded-md border border-sky-500/30 bg-sky-500/5 p-3">
            <p className="text-xs text-muted-foreground">{t("Wiki pages")}</p>
            <p className="text-lg font-semibold">{pages.length}</p>
          </div>
          <div className="rounded-md border p-3">
            <p className="text-xs text-muted-foreground">{t("Selected page")}</p>
            <p className="truncate text-lg font-semibold">{selectedPage?.title ?? "--"}</p>
          </div>
          <div className="rounded-md border p-3">
            <p className="text-xs text-muted-foreground">{t("Format")}</p>
            <p className="text-lg font-semibold">{selectedPage?.format ?? "markdown"}</p>
          </div>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isComposerOpen ? "secondary" : "outline"}
            disabled={!canWriteWiki}
            onClick={() => setComposerOpen((current) => !current)}
          >
            <Plus className="size-4" />
            {isComposerOpen ? t("Hide new wiki page form") : t("New wiki page")}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadPages()}>
            <RefreshCw className="size-4" />
            {t("Reload")}
          </Button>
        </div>

        {!canWriteWiki ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect wiki pages, but cannot edit them.")}</AlertDescription>
          </Alert>
        ) : null}

        {isComposerOpen ? (
          <form className="space-y-3 rounded-md border p-3" onSubmit={submitCreatePage}>
            <div className="grid gap-3 md:grid-cols-[1fr_220px_180px]">
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="wiki-title">
                  {t("Wiki page title")}
                </label>
                <Input
                  id="wiki-title"
                  value={newTitle}
                  onChange={(event) => setNewTitle(event.target.value)}
                  placeholder={t("Getting Started")}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="wiki-slug">
                  {t("Slug optional")}
                </label>
                <Input
                  id="wiki-slug"
                  value={newSlug}
                  onChange={(event) => setNewSlug(event.target.value)}
                  placeholder="getting-started"
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="wiki-author-id">
                  {t("Author user ID")}
                </label>
                <Input
                  id="wiki-author-id"
                  value={authorUserId}
                  onChange={(event) => setAuthorUserId(event.target.value)}
                  placeholder="1"
                />
              </div>
            </div>
            <Textarea
              className="min-h-44"
              value={newContent}
              onChange={(event) => setNewContent(event.target.value)}
              placeholder="# Getting Started"
            />
            <div className="flex justify-end">
              <Button type="submit" disabled={!canWriteWiki || isCreatingPage}>
                {isCreatingPage ? t("Creating wiki page...") : t("Create wiki page")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="grid gap-4 lg:grid-cols-[280px_1fr]">
          <div className="space-y-2 rounded-md border p-2">
            {isLoadingPages ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading wiki pages...")}</p> : null}
            {!isLoadingPages && pages.length === 0 ? (
              <p className="px-2 py-2 text-sm text-muted-foreground">{t("No wiki pages found.")}</p>
            ) : null}
            {pages.map((page) => (
              <button
                key={page.slug}
                type="button"
                className={`w-full rounded-md border p-3 text-left transition hover:bg-muted/40 ${selectedPage?.slug === page.slug ? "border-primary bg-primary/5" : ""}`}
                onClick={() => setSelectedSlug(page.slug)}
              >
                <div className="flex items-start gap-2">
                  <BookOpen className="mt-0.5 size-4 text-muted-foreground" />
                  <div className="min-w-0">
                    <p className="truncate font-medium">{page.title}</p>
                    <p className="truncate text-xs text-muted-foreground">{page.slug}</p>
                  </div>
                </div>
                <p className="mt-2 text-xs text-muted-foreground">
                  {page.updated_at ? formatRelativeTime(page.updated_at) : t("Not edited yet")}
                </p>
              </button>
            ))}
          </div>

          <div className="rounded-md border">
            {selectedPage ? (
              <form className="space-y-4 p-4" onSubmit={submitUpdatePage}>
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0 space-y-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <Badge variant="secondary">{selectedPage.slug}</Badge>
                      <Badge variant="outline">{selectedPage.format}</Badge>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {t("Last edited by")}: {selectedPage.last_edited_by_user_id || "--"}
                    </p>
                  </div>
                  <ConfirmAction
                    title={selectedPage ? t("Delete wiki page \"{title}\"?").replace("{title}", selectedPage.title) : t("Delete")}
                    description={t("This action cannot be undone.")}
                    confirmLabel={t("Delete")}
                    cancelLabel={t("Cancel")}
                    onConfirm={() => void submitDeletePage()}
                  >
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={!canWriteWiki || isDeletingPage}
                    >
                      <Trash2 className="size-4" />
                      {t("Delete")}
                    </Button>
                  </ConfirmAction>
                </div>

                <div className="grid gap-3 md:grid-cols-[1fr_180px]">
                  <div className="space-y-1">
                    <label className="text-xs font-medium text-muted-foreground" htmlFor="wiki-draft-title">
                      {t("Wiki page title")}
                    </label>
                    <Input
                      id="wiki-draft-title"
                      value={draftTitle}
                      disabled={!canWriteWiki}
                      onChange={(event) => setDraftTitle(event.target.value)}
                    />
                  </div>
                  <div className="space-y-1">
                    <label className="text-xs font-medium text-muted-foreground" htmlFor="wiki-editor-id">
                      {t("Editor user ID")}
                    </label>
                    <Input
                      id="wiki-editor-id"
                      value={editorUserId}
                      disabled={!canWriteWiki}
                      onChange={(event) => setEditorUserId(event.target.value)}
                      placeholder="1"
                    />
                  </div>
                </div>

                <div className="grid gap-4 xl:grid-cols-2">
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <Edit3 className="size-4" />
                      {t("Write")}
                    </div>
                    <Textarea
                      className="min-h-80"
                      value={draftContent}
                      disabled={!canWriteWiki}
                      onChange={(event) => setDraftContent(event.target.value)}
                    />
                  </div>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <Eye className="size-4" />
                      {t("Preview")}
                    </div>
                    <article
                      className="markdown-body min-h-80 rounded-md border bg-muted/20 p-4"
                      dangerouslySetInnerHTML={{ __html: previewHtml }}
                    />
                  </div>
                </div>

                <div className="flex justify-end">
                  <Button type="submit" disabled={!canWriteWiki || isUpdatingPage}>
                    {isUpdatingPage ? t("Saving wiki page...") : t("Save wiki page")}
                  </Button>
                </div>
              </form>
            ) : (
              <div className="flex min-h-64 items-center justify-center p-6 text-sm text-muted-foreground">
                {t("Select or create a wiki page.")}
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const resolveWikiList = (payload: unknown): RawWikiPage[] => {
  if (Array.isArray(payload)) {
    return resolveRecordArray(payload);
  }
  return isRecord(payload) ? resolveRecordArray(payload.body ?? payload.Body) : [];
};

const resolveWikiPage = (payload: unknown): RawWikiPage => {
  if (isRecord(payload)) {
    const nested = payload.body ?? payload.Body;
    if (isRecord(nested)) {
      return nested;
    }
    return payload;
  }
  return {};
};

const normalizeWikiPage = (raw: RawWikiPage): RepositoryWikiPageView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  slug: normalizeString(raw.slug ?? raw.Slug),
  title: normalizeString(raw.title ?? raw.Title) || "Untitled",
  content: normalizeString(raw.content ?? raw.Content),
  format: normalizeString(raw.format ?? raw.Format) || "markdown",
  author_user_id: normalizeString(raw.author_user_id ?? raw.AuthorUserID),
  last_edited_by_user_id: normalizeString(raw.last_edited_by_user_id ?? raw.LastEditedByUserID),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const parseRequiredUserID = (value: string): number | null => {
  const parsed = Number.parseInt(value.trim(), 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
};

const parseOptionalUserID = (value: string): number | null => {
  if (!value.trim()) {
    return 0;
  }
  return parseRequiredUserID(value);
};
