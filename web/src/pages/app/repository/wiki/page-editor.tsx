import { Edit3, Eye, Trash2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type { RepositoryWikiPageView } from "@/pages/types";

interface WikiPageEditorProps {
  page: RepositoryWikiPageView;
  title: string;
  content: string;
  previewHtml: string;
  canWrite: boolean;
  isUpdating: boolean;
  isDeleting: boolean;
  t: (text: string) => string;
  onTitleChange: (value: string) => void;
  onContentChange: (value: string) => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
  onDelete: () => void;
}

export const WikiPageEditor = ({
  page,
  title,
  content,
  previewHtml,
  canWrite,
  isUpdating,
  isDeleting,
  t,
  onTitleChange,
  onContentChange,
  onSubmit,
  onDelete,
}: WikiPageEditorProps): JSX.Element => (
  <form className="flex flex-col gap-4 p-4" onSubmit={onSubmit}>
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div className="flex min-w-0 flex-col gap-1">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="secondary">{page.slug}</Badge>
          <Badge variant="outline">{page.format}</Badge>
        </div>
        <p className="text-xs text-muted-foreground">
          {t("Last edited by")}: {page.last_edited_by_user_id || "--"}
        </p>
      </div>
      <ConfirmAction
        title={t('Delete wiki page "{title}"?').replace("{title}", page.title)}
        description={t("This action cannot be undone.")}
        confirmLabel={t("Delete")}
        cancelLabel={t("Cancel")}
        onConfirm={onDelete}
      >
        <Button type="button" variant="outline" size="sm" disabled={!canWrite || isDeleting}>
          <Trash2 data-icon="inline-start" />
          {t("Delete")}
        </Button>
      </ConfirmAction>
    </div>
    <div className="flex flex-col gap-1">
      <Label htmlFor="wiki-draft-title" className="text-xs text-muted-foreground">
        {t("Wiki page title")}
      </Label>
      <Input
        id="wiki-draft-title"
        value={title}
        disabled={!canWrite}
        onChange={(event) => onTitleChange(event.target.value)}
      />
    </div>
    <div className="grid gap-4 xl:grid-cols-2">
      <div className="flex flex-col gap-2">
        <Label htmlFor="wiki-draft-content" className="flex items-center gap-2 text-sm font-medium">
          <Edit3 className="size-4" />
          {t("Wiki page content")}
        </Label>
        <Textarea
          id="wiki-draft-content"
          className="min-h-80"
          value={content}
          disabled={!canWrite}
          onChange={(event) => onContentChange(event.target.value)}
        />
      </div>
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2 text-sm font-medium">
          <Eye className="size-4" />
          {t("Preview")}
        </div>
        <article
          className="markdown-body min-h-80 border bg-muted/20 p-4"
          dangerouslySetInnerHTML={{ __html: previewHtml }}
        />
      </div>
    </div>
    <div className="flex justify-end">
      <Button type="submit" size="sm" disabled={!canWrite || isUpdating}>
        {isUpdating ? t("Saving wiki page...") : t("Save wiki page")}
      </Button>
    </div>
  </form>
);
