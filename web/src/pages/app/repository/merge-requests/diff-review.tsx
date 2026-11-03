import { useMemo, useState, type FormEvent } from "react";
import { MessageSquarePlus, Send } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { formatRelativeTime } from "../issues-utils";
import { isRecord, normalizeNumber, normalizeString, resolveArrayPayload } from "../repository-normalizers";

export interface InlineReviewPosition {
  file: string;
  line: number;
  column?: number;
}

export interface InlineReviewComment {
  id: string;
  authorID: string;
  body: string;
  createdAt: string;
  position: InlineReviewPosition | null;
}

export interface InlineReviewThread {
  id: string;
  resolved: boolean;
  comments: InlineReviewComment[];
}

interface DiffReviewProps {
  diff: string;
  threads: InlineReviewThread[];
  t: (text: string) => string;
  canComment: boolean;
  isSubmitting: boolean;
  userLabels: ReadonlyMap<string, string>;
  onSubmit: (body: string, position: InlineReviewPosition, discussionID?: string) => Promise<boolean>;
}

export interface DiffLine {
  text: string;
  kind: "context" | "added" | "removed" | "meta";
  file: string;
  line: number | null;
}

export const resolveInlineReviewThreads = (payload: unknown): InlineReviewThread[] =>
  resolveArrayPayload<unknown>(payload).filter(isRecord).map((raw) => ({
    id: normalizeString(raw.id ?? raw.ID),
    resolved: raw.resolved === true || raw.Resolved === true,
    comments: resolveArrayPayload<unknown>(raw.comments ?? raw.Comments).filter(isRecord).map((comment) => ({
      id: normalizeString(comment.id ?? comment.ID),
      authorID: normalizeString(comment.author_user_id ?? comment.AuthorUserID),
      body: normalizeString(comment.body ?? comment.Body),
      createdAt: normalizeString(comment.created_at ?? comment.CreatedAt),
      position: normalizePosition(comment.position ?? comment.Position),
    })),
  })).filter((thread) => thread.id.length > 0);

const normalizePosition = (value: unknown): InlineReviewPosition | null => {
  if (!isRecord(value)) return null;
  const file = normalizeString(value.file ?? value.File);
  const line = normalizeNumber(value.line ?? value.Line);
  if (!file || line < 1) return null;
  const column = normalizeNumber(value.column ?? value.Column);
  return { file, line, ...(column > 0 ? { column } : {}) };
};

export const parseDiff = (diff: string): DiffLine[] => {
  const lines: DiffLine[] = [];
  let file = "";
  let oldLine = 0;
  let newLine = 0;
  for (const text of diff.split("\n")) {
    if (text.startsWith("diff --git ")) {
      const pathParts = text.split(" b/");
      const targetPath = pathParts[pathParts.length - 1];
      if (targetPath) file = targetPath;
      lines.push({ text, kind: "meta", file, line: null });
      continue;
    }
    if (text.startsWith("+++ b/")) { file = text.slice(6); lines.push({ text, kind: "meta", file, line: null }); continue; }
    if (text.startsWith("+++ /dev/null")) { lines.push({ text, kind: "meta", file, line: null }); continue; }
    if (text.startsWith("--- a/") || text.startsWith("--- /dev/null")) {
      lines.push({ text, kind: "meta", file, line: null });
      continue;
    }
    const hunk = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(text);
    if (hunk) {
      oldLine = Number(hunk[1]);
      newLine = Number(hunk[2]);
      lines.push({ text, kind: "meta", file, line: null });
      continue;
    }
    if (text.startsWith("+") && !text.startsWith("+++")) { lines.push({ text, kind: "added", file, line: newLine++ }); continue; }
    if (text.startsWith("-") && !text.startsWith("---")) {
      lines.push({ text, kind: "removed", file, line: oldLine++ });
      continue;
    }
    if (text.startsWith("\\ No newline")) { lines.push({ text, kind: "meta", file, line: null }); continue; }
    lines.push({ text, kind: "context", file, line: oldLine > 0 ? newLine++ : null });
    if (oldLine > 0) oldLine++;
  }
  return lines;
};

export const DiffReview = ({
  diff, threads, t, canComment, isSubmitting, userLabels, onSubmit,
}: DiffReviewProps): JSX.Element => {
  const lines = useMemo(() => parseDiff(diff), [diff]);
  const [activeAnchor, setActiveAnchor] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const anchorKey = (file: string, line: number) => `${file}:${line}`;
  const handleSubmit = async (event: FormEvent<HTMLFormElement>, position: InlineReviewPosition, discussionID?: string) => {
    event.preventDefault();
    const body = draft.trim();
    if (!body || isSubmitting) return;
    if (await onSubmit(body, position, discussionID)) {
      setDraft("");
      setActiveAnchor(null);
    }
  };

  return (
    <div className="overflow-auto bg-background font-mono text-xs leading-6">
      {lines.map((line, index) => {
        const key = `${index}:${line.file}`;
        const attached = line.line === null ? [] : threads.filter((thread) =>
          thread.comments.some((comment) => comment.position?.file === line.file && comment.position.line === line.line));
        const anchor = line.line === null ? "" : anchorKey(line.file, line.line);
        const tone = line.kind === "added"
          ? "bg-success/10 text-foreground"
          : line.kind === "removed"
            ? "bg-destructive/10 text-foreground"
            : line.kind === "meta"
              ? "bg-muted text-muted-foreground"
              : "";
        return (
          <div key={key}>
            <div className={`group flex min-w-max ${tone}`}>
              <span className="w-12 shrink-0 select-none px-2 text-right text-muted-foreground">{line.line ?? ""}</span>
              <code className="whitespace-pre px-3">{line.text}</code>
              {line.line !== null && line.file && canComment ? (
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="mx-1 size-6 opacity-0 focus-visible:opacity-100 group-hover:opacity-100"
                  aria-label={t("Add inline comment")}
                  onClick={() => { setActiveAnchor(anchor); setDraft(""); }}
                >
                  <MessageSquarePlus className="size-3.5" />
                </Button>
              ) : null}
            </div>
            {attached.map((thread) => (
              <InlineThreadView
                key={thread.id}
                thread={thread}
                userLabels={userLabels}
                t={t}
                isSubmitting={isSubmitting}
                onReply={(body, position) => onSubmit(body, position, thread.id)}
              />
            ))}
            {activeAnchor === anchor && line.line !== null ? (
              <form
                className="border-y border-border bg-muted/30 p-3"
                onSubmit={(event) => void handleSubmit(event, { file: line.file, line: line.line! })}
              >
                <label className="sr-only" htmlFor={`inline-comment-${index}`}>{t("Add inline comment")}</label>
                <Textarea
                  id={`inline-comment-${index}`}
                  autoFocus
                  value={draft}
                  onChange={(event) => setDraft(event.target.value)}
                  placeholder={t("Write a comment on this line...")}
                  className="min-h-20 bg-background font-sans"
                />
                <div className="mt-2 flex justify-end gap-2">
                  <Button type="button" size="sm" variant="ghost" onClick={() => setActiveAnchor(null)}>{t("Cancel")}</Button>
                  <Button type="submit" size="sm" disabled={!draft.trim() || isSubmitting}>
                    <Send data-icon="inline-start" />
                    {isSubmitting ? t("Sending...") : t("Add comment")}
                  </Button>
                </div>
              </form>
            ) : null}
          </div>
        );
      })}
      {!diff.trim() ? <p className="p-4 font-sans text-sm text-muted-foreground">{t("No diff available.")}</p> : null}
    </div>
  );
};

const InlineThreadView = ({ thread, userLabels, t, isSubmitting, onReply }: {
  thread: InlineReviewThread;
  userLabels: ReadonlyMap<string, string>;
  t: DiffReviewProps["t"];
  isSubmitting: boolean;
  onReply: (body: string, position: InlineReviewPosition) => Promise<boolean>;
}): JSX.Element => {
  const [reply, setReply] = useState("");
  const position = thread.comments.find((comment) => comment.position)?.position;
  const submitReply = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!position || !reply.trim() || isSubmitting) return;
    if (await onReply(reply.trim(), position)) setReply("");
  };
  return (
    <section className="border-y border-border bg-muted/30 px-4 py-3 font-sans" aria-label={t("Inline discussion")}>
      <div className="mb-2 flex items-center gap-2">
        <Badge variant={thread.resolved ? "secondary" : "outline"}>
          {thread.resolved ? t("Resolved") : t("Open discussion")}
        </Badge>
        <span className="text-xs text-muted-foreground">{thread.comments.length} {t("comment(s)")}</span>
      </div>
      <div className="space-y-3">
        {thread.comments.map((comment) => (
          <article key={comment.id} className="border-l-2 border-primary/40 pl-3">
            <p className="text-xs font-medium">{userLabels.get(comment.authorID) ?? comment.authorID}</p>
            <p className="mt-1 whitespace-pre-wrap text-sm">{comment.body}</p>
            {comment.createdAt ? (
              <time className="mt-1 block text-[11px] text-muted-foreground">{formatRelativeTime(comment.createdAt)}</time>
            ) : null}
          </article>
        ))}
      </div>
      {position && !thread.resolved ? (
        <form className="mt-3 flex flex-col gap-2" onSubmit={(event) => void submitReply(event)}>
          <label className="sr-only" htmlFor={`inline-reply-${thread.id}`}>{t("Reply to discussion")}</label>
          <Textarea
            id={`inline-reply-${thread.id}`}
            value={reply}
            onChange={(event) => setReply(event.target.value)}
            placeholder={t("Reply to discussion...")}
            className="min-h-16 bg-background"
          />
          <Button type="submit" size="sm" className="self-end" disabled={!reply.trim() || isSubmitting}>
            {isSubmitting ? t("Sending...") : t("Reply")}
          </Button>
        </form>
      ) : null}
    </section>
  );
};
