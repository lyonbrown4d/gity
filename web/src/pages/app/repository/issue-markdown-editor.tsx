import { useEffect, useRef, useState, type ChangeEvent } from "react";
import {
  commandsCtx,
  defaultValueCtx,
  Editor,
  editorViewCtx,
  parserCtx,
  rootCtx,
} from "@milkdown/core";
import { listener, listenerCtx } from "@milkdown/plugin-listener";
import { commonmark } from "@milkdown/preset-commonmark";
import { gfm } from "@milkdown/preset-gfm";
import { Milkdown, MilkdownProvider, useEditor } from "@milkdown/react";
import { nord } from "@milkdown/theme-nord";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { apiRequest } from "@/lib/api";
import type { IssueAttachmentUploadView } from "@/pages/types";
import { renderIssueMarkdown } from "./issue-markdown";
import { IssueMarkdownToolbar, type IssueEditorToolbarAction } from "./issue-markdown-toolbar";
import { appendMarkdown, resolveToolbarRequest } from "./issue-markdown-editor-actions";
interface IssueMarkdownEditorProps {
  organizationId: string;
  repoId: string;
  issueId?: string | null;
  t: (text: string) => string;
  value: string;
  placeholder: string;
  editorHeight?: number;
  onChange: (value: string) => void;
  onError: (message: string | null) => void;
}

interface MilkdownEditorCoreProps extends IssueMarkdownEditorProps {
  onUploadFiles: (files: File[]) => Promise<void>;
}
const IssueMilkdownCore = ({
  organizationId,
  repoId,
  t,
  value,
  placeholder,
  editorHeight,
  onChange,
  onError,
  onUploadFiles,
}: MilkdownEditorCoreProps): JSX.Element => {
  const [mode, setMode] = useState<"write" | "preview">("write");
  const [previewHtml, setPreviewHtml] = useState<string>("");
  const [isRenderingPreview, setRenderingPreview] = useState(false);
  const [isUploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const syncingRef = useRef(false);
  const markdownRef = useRef(value);

  const { loading, get } = useEditor((root) => {
    const editor = Editor.make()
      .config(nord)
      .config((ctx) => {
        ctx.set(rootCtx, root);
        ctx.set(defaultValueCtx, value || "");
      })
      .config((ctx) => {
        ctx.get(listenerCtx).markdownUpdated((_ctx, markdown) => {
          markdownRef.current = markdown;
          if (!syncingRef.current) {
            onChange(markdown);
          }
        });
      })
      .use(listener)
      .use(commonmark)
      .use(gfm);
    return editor;
  }, []);

  const runToolbarAction = (action: IssueEditorToolbarAction) => {
    onError(null);
    const editor = get();
    if (!editor) {
      onError(t("Editor is still initializing"));
      return;
    }
    const request = resolveToolbarRequest(action, t);
    if (!request) {
      return;
    }

    try {
      editor.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        if (request.type === "insert_text") {
          const { from, to } = view.state.selection;
          view.dispatch(view.state.tr.insertText(request.text, from, to));
          view.focus();
          return;
        }
        const command = ctx.get(commandsCtx);
        command.call(request.key, request.payload);
        view.focus();
      });
    } catch {
      onError(t("Failed to run editor command"));
    }
  };

  useEffect(() => {
    if (mode !== "preview") {
      return;
    }
    let active = true;
    setRenderingPreview(true);
    renderIssueMarkdown(value || "*No content*", organizationId, repoId)
      .then((html) => {
        if (active) {
          setPreviewHtml(html);
        }
      })
      .catch(() => {
        if (active) {
          setPreviewHtml("<p>Failed to render markdown preview.</p>");
        }
      })
      .finally(() => {
        if (active) {
          setRenderingPreview(false);
        }
      });
    return () => {
      active = false;
    };
  }, [mode, value, organizationId, repoId]);

  useEffect(() => {
    if (loading) {
      return;
    }
    const editor = get();
    if (!editor) {
      return;
    }
    if (value === markdownRef.current) {
      return;
    }
    syncingRef.current = true;
    editor.action((ctx) => {
      const parser = ctx.get(parserCtx);
      const view = ctx.get(editorViewCtx);
      const doc = parser(value || "");
      if (!doc) {
        return;
      }
      const tr = view.state.tr.replaceWith(0, view.state.doc.content.size, doc.content);
      view.dispatch(tr);
    });
    markdownRef.current = value;
    queueMicrotask(() => {
      syncingRef.current = false;
    });
  }, [value, loading, get]);

  const handlePickFiles = async (event: ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? []);
    if (files.length === 0) {
      return;
    }
    setUploading(true);
    try {
      await onUploadFiles(files);
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  const isEditorReady = !loading && Boolean(get());

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="inline-flex overflow-hidden rounded-md border">
          <Button
            type="button"
            size="sm"
            variant={mode === "write" ? "default" : "ghost"}
            className="rounded-none"
            onClick={() => setMode("write")}
          >
            {t("Write")}
          </Button>
          <Button
            type="button"
            size="sm"
            variant={mode === "preview" ? "default" : "ghost"}
            className="rounded-none"
            onClick={() => setMode("preview")}
          >
            {t("Preview")}
          </Button>
        </div>
        <div className="flex items-center gap-2">
          <input
            ref={fileInputRef}
            type="file"
            className="hidden"
            multiple
            onChange={(event) => void handlePickFiles(event)}
          />
          <Button
            type="button"
            size="sm"
            variant="outline"
            disabled={isUploading}
            onClick={() => fileInputRef.current?.click()}
          >
            {isUploading ? t("Uploading...") : t("Upload files")}
          </Button>
        </div>
      </div>

      {mode === "write" ? (
        <div className="issue-editor-shell">
          <IssueMarkdownToolbar
            t={t}
            disabled={!isEditorReady || isUploading}
            onAction={(action) => runToolbarAction(action)}
          />
          <div style={{ minHeight: `${Math.max(editorHeight ?? 220, 160)}px` }} className="issue-milkdown">
            <Milkdown />
          </div>
          {!isEditorReady ? (
            <p className="px-3 pb-2 text-xs text-muted-foreground">{t("Editor is still initializing")}</p>
          ) : null}
          {!value.trim() ? <p className="px-3 pb-2 text-xs text-muted-foreground">{placeholder}</p> : null}
        </div>
      ) : (
        <div className="min-h-[120px] rounded-md border bg-muted/20 px-3 py-2">
          {isRenderingPreview ? (
            <p className="text-xs text-muted-foreground">{t("Rendering preview...")}</p>
          ) : (
            <article className="markdown-body" dangerouslySetInnerHTML={{ __html: previewHtml }} />
          )}
        </div>
      )}
    </div>
  );
};

export const IssueMarkdownEditor = (props: IssueMarkdownEditorProps): JSX.Element => {
  const { repoId, issueId, onError, onChange, value } = props;
  const [uploadedFiles, setUploadedFiles] = useState<IssueAttachmentUploadView[]>([]);

  const uploadFile = async (file: File) => {
    const form = new FormData();
    form.append("file", file);
    const query = new URLSearchParams();
    if (issueId) {
      query.set("issue_id", issueId);
    }
    const suffix = query.toString() ? `?${query.toString()}` : "";
    return apiRequest<IssueAttachmentUploadView>(`/repos/${repoId}/issues/attachments${suffix}`, {
      method: "POST",
      body: form,
    });
  };

  const handleUploadFiles = async (files: File[]) => {
    onError(null);
    try {
      const uploaded: IssueAttachmentUploadView[] = [];
      let next = value;
      for (const file of files) {
        const item = await uploadFile(file);
        uploaded.push(item);
        next = appendMarkdown(next, item.markdown);
      }
      setUploadedFiles((current) => [...uploaded, ...current].slice(0, 8));
      onChange(next);
    } catch (error) {
      onError(error instanceof Error ? error.message : "Failed to upload attachment");
    }
  };

  return (
    <MilkdownProvider>
      <IssueMilkdownCore {...props} onUploadFiles={handleUploadFiles} />
      {uploadedFiles.length > 0 ? (
        <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          {uploadedFiles.map((item) => (
            <Badge key={item.object_key} variant="outline">
              {item.file_name}
            </Badge>
          ))}
        </div>
      ) : null}
    </MilkdownProvider>
  );
};
