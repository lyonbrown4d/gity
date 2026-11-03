import { useEffect, useMemo, useState } from "react";
import { BookOpen, RefreshCw } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataList, DataRow } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { Toolbar } from "@/components/surface/toolbar";
import { cn } from "@/lib/utils";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import { WikiCreateDialog } from "./wiki/create-dialog";
import { normalizeWikiPage, resolveWikiList, resolveWikiPage, type RawWikiPage } from "./wiki/model";
import { WikiPageEditor } from "./wiki/page-editor";
import type { RepositoryPermissions } from "./repository-permissions";
import { renderMarkdown } from "./repository-utils";

interface RepositoryWikiTabProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryWikiTab = ({ repoId, permissions, t, onError }: RepositoryWikiTabProps): JSX.Element => {
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

  const pages = useMemo(
    () => resolveWikiList(pagesQuery.result.data).map(normalizeWikiPage),
    [pagesQuery.result.data],
  );
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
  const selectedPage = useMemo(
    () => pages.find((item) => item.slug === selectedSlug) ?? null,
    [pages, selectedSlug],
  );
  const [draftTitle, setDraftTitle] = useState("");
  const [draftContent, setDraftContent] = useState("");
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

  const createPage = async (title: string, slug: string, content: string): Promise<string> => {
    onError(null);
    try {
      const response = await createWikiPage({
        url: `/projects/${repoId}/wiki/pages`,
        method: "post",
        values: { title, ...(slug ? { slug } : {}), content, format: "markdown" },
      });
      const created = normalizeWikiPage(resolveWikiPage(response.data));
      setSelectedSlug(created.slug || null);
      await loadPages();
      return created.slug;
    } catch (error) {
      throw new Error(extractErrorMessage(error));
    }
  };

  const submitUpdatePage = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedPage) {
      return;
    }
    const title = draftTitle.trim();
    if (!title) {
      onError(t("Wiki page title is required"));
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
    <Section
      title={t("Wiki")}
      description={t("Maintain project wiki pages with markdown content.")}
      actions={
        <Toolbar>
          <Badge variant={canWriteWiki ? "secondary" : "outline"}>
            {canWriteWiki ? t("Editing enabled") : t("Read only")}
          </Badge>
          <WikiCreateDialog canWrite={canWriteWiki} busy={isCreatingPage} t={t} onCreate={createPage} />
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadPages()}>
            <RefreshCw data-icon="inline-start" />
            {t("Reload")}
          </Button>
        </Toolbar>
      }
      flush
    >
      <div className="grid divide-y divide-border border-b border-border sm:grid-cols-3 sm:divide-x sm:divide-y-0">
        <WikiStat label={t("Wiki pages")} value={pages.length} />
        <WikiStat label={t("Selected page")} value={selectedPage?.title ?? "--"} />
        <WikiStat label={t("Format")} value={selectedPage?.format ?? "markdown"} />
      </div>

      {!canWriteWiki ? (
        <div className="border-b border-border px-4 py-3">
          <Alert>
            <AlertTitle>{t("Wiki is read-only")}</AlertTitle>
            <AlertDescription>{t("Your current project role can inspect wiki pages, but cannot edit them.")}</AlertDescription>
          </Alert>
        </div>
      ) : null}

      <div className="grid lg:grid-cols-[280px_minmax(0,1fr)]">
        <div className="min-w-0 border-b border-border lg:border-b-0 lg:border-r">
          {isLoadingPages ? <p className="px-4 py-3 text-sm text-muted-foreground">{t("Loading wiki pages...")}</p> : null}
          {!isLoadingPages && pages.length === 0 ? (
            <div className="px-4 py-6">
              <WikiEmptyState
                title={t("No wiki pages found.")}
                description={canWriteWiki ? t("Create the first wiki page to document this project.") : t("Wiki pages will appear here after maintainers create them.")}
              />
            </div>
          ) : null}
          {pages.length > 0 ? (
            <DataList>
              {pages.map((page) => (
                <DataRow key={page.slug} className={cn("p-0", selectedPage?.slug === page.slug && "bg-primary/5")}>
                  <button
                    type="button"
                    className="flex w-full items-center gap-2 px-4 py-2.5 text-left"
                    onClick={() => setSelectedSlug(page.slug)}
                  >
                    <BookOpen className="size-3.5 shrink-0 text-muted-foreground" />
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-sm font-medium">{page.title}</span>
                      <span className="mt-0.5 block truncate text-xs text-muted-foreground">
                        {page.slug} · {page.updated_at ? formatRelativeTime(page.updated_at) : t("Not edited yet")}
                      </span>
                    </span>
                  </button>
                </DataRow>
              ))}
            </DataList>
          ) : null}
        </div>

        <div className="min-w-0">
          {selectedPage ? (
            <WikiPageEditor page={selectedPage} title={draftTitle} content={draftContent} previewHtml={previewHtml} canWrite={canWriteWiki} isUpdating={isUpdatingPage} isDeleting={isDeletingPage} t={t} onTitleChange={setDraftTitle} onContentChange={setDraftContent} onSubmit={submitUpdatePage} onDelete={() => void submitDeletePage()} />
          ) : (
            <div className="px-4 py-10">
              <WikiEmptyState
                title={t("Select or create a wiki page.")}
                description={canWriteWiki ? t("Create a wiki page or select one from the list to edit markdown content.") : t("Select a wiki page from the list to preview its content.")}
              />
            </div>
          )}
        </div>
      </div>
    </Section>
  );
};

const WikiStat = ({ label, value }: { label: string; value: number | string }) => (
  <div className="px-4 py-2.5">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="mt-0.5 truncate text-sm font-semibold">{value}</p>
  </div>
);

const WikiEmptyState = ({ title, description }: { title: string; description: string }) => (
  <div className="flex flex-col items-center justify-center gap-2 text-center">
    <BookOpen className="size-4 text-muted-foreground" />
    <div className="flex flex-col gap-1">
      <p className="text-sm font-medium">{title}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
  </div>
);
