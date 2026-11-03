import { useState } from "react";
import { Plus } from "lucide-react";
import { FormDialog } from "@/components/common/form-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";

interface WikiCreateDialogProps {
  canWrite: boolean;
  busy: boolean;
  t: (text: string) => string;
  onCreate: (title: string, slug: string, content: string) => Promise<string>;
}

export const WikiCreateDialog = ({ canWrite, busy, t, onCreate }: WikiCreateDialogProps): JSX.Element => {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [slug, setSlug] = useState("");
  const [content, setContent] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setSubmitting] = useState(false);
  const isBusy = busy || isSubmitting;

  const submit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (isBusy) return;
    const normalizedTitle = title.trim();
    if (!normalizedTitle) {
      setError(t("Wiki page title is required"));
      return;
    }
    setError(null);
    setSubmitting(true);
    try {
      await onCreate(normalizedTitle, slug.trim(), content);
      setTitle("");
      setSlug("");
      setContent("");
      setOpen(false);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : t("Failed to create wiki page."));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <FormDialog
      open={open}
      onOpen={() => {
        setError(null);
        setOpen(true);
      }}
      onClose={() => setOpen(false)}
      busy={isBusy}
      title={t("New wiki page")}
      description={t("Give the page a title and write its markdown content. The page URL is based on the title.")}
      trigger={
        <Button type="button" size="sm" disabled={!canWrite}>
          <Plus data-icon="inline-start" />
          {t("New wiki page")}
        </Button>
      }
    >
      <form className="flex flex-col gap-4" onSubmit={(event) => void submit(event)}>
        <fieldset disabled={isBusy} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <Label htmlFor="wiki-title">{t("Wiki page title")}</Label>
            <Input
              id="wiki-title"
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              placeholder={t("Getting Started")}
              required
            />
          </div>
          <details className="rounded-md border border-border p-3">
            <summary className="cursor-pointer text-sm font-medium">{t("Advanced options")}</summary>
            <div className="mt-3 flex flex-col gap-1">
              <Label htmlFor="wiki-slug">{t("Custom URL slug")}</Label>
              <Input
                id="wiki-slug"
                value={slug}
                onChange={(event) => setSlug(event.target.value)}
                placeholder={t("Leave empty to use the title")}
              />
            </div>
          </details>
          <div className="flex flex-col gap-1">
            <Label htmlFor="wiki-content">{t("Content")}</Label>
            <Textarea
              id="wiki-content"
              className="min-h-44"
              value={content}
              onChange={(event) => setContent(event.target.value)}
              placeholder="# Getting Started"
            />
          </div>
        </fieldset>
        {error ? (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        ) : null}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={() => setOpen(false)} disabled={isBusy}>
            {t("Cancel")}
          </Button>
          <Button type="submit" disabled={isBusy || !canWrite}>
            {isBusy ? t("Creating wiki page...") : t("Create wiki page")}
          </Button>
        </div>
      </form>
    </FormDialog>
  );
};
