import { useEffect, useRef } from "react";
import Editor, { type Monaco } from "@/components/code-editor";
import { BookOpen, FileCode2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { RepositoryBlobView, RepositoryView } from "@/pages/types";
import { RepositoryEmptyState } from "../repository-empty-state";
import { detectLanguage } from "../repository-utils";
import { registerEditorTheme } from "@/lib/monaco-theme";

interface SearchableEditor {
  setPosition: (position: { lineNumber: number; column: number }) => void;
  revealLineInCenter: (lineNumber: number) => void;
  focus: () => void;
  deltaDecorations: (
    oldDecorations: string[],
    newDecorations: Array<{
      range: {
        startLineNumber: number;
        startColumn: number;
        endLineNumber: number;
        endColumn: number;
      };
      options: { isWholeLine?: boolean; className: string };
    }>,
  ) => string[];
}

interface RepositorySourceCanvasProps {
  t: (text: string) => string;
  repository: RepositoryView;
  selectedBlob: RepositoryBlobView | null;
  readmePreview: string | null;
  readmePath: string | null;
  isEmptyRepository: boolean;
  editorTheme: string;
  searchLine: number;
  searchColumn: number;
  searchMatchLength: number;
  searchJumpRequest: number;
  searchTargetPath: string;
  onOpenFile: (path: string) => void;
}

export const RepositorySourceCanvas = ({
  t, repository, selectedBlob, readmePreview, readmePath, isEmptyRepository,
  editorTheme, searchLine, searchColumn, searchMatchLength, searchJumpRequest,
  searchTargetPath, onOpenFile,
}: RepositorySourceCanvasProps): JSX.Element => {
  const editorRef = useRef<SearchableEditor | null>(null);
  const monacoRef = useRef<Monaco | null>(null);
  const decorationsRef = useRef<string[]>([]);

  const jumpToSearchLine = () => {
    const editor = editorRef.current;
    if (!editor) {
      decorationsRef.current = [];
      return;
    }
    if (!selectedBlob || !searchTargetPath || selectedBlob.path !== searchTargetPath) {
      if (decorationsRef.current.length > 0) {
        decorationsRef.current = editor.deltaDecorations(decorationsRef.current, []);
      }
      return;
    }
    const lines = selectedBlob.content.split("\n");
    const targetLine = Math.min(Math.max(searchLine, 1), Math.max(lines.length, 1));
    const targetText = lines[targetLine - 1] ?? "";
    const targetColumn = Math.min(Math.max(searchColumn, 1), targetText.length + 1);
    const matchLength = Math.max(0, Math.floor(searchMatchLength));
    editor.setPosition({ lineNumber: targetLine, column: targetColumn });
    editor.revealLineInCenter(targetLine);
    editor.focus();
    const decorations = [{
      range: { startLineNumber: targetLine, startColumn: 1, endLineNumber: targetLine, endColumn: Math.max(targetText.length + 1, 1) },
      options: { isWholeLine: true, className: "search-jump-line-decoration" },
    }];
    if (matchLength > 0) {
      decorations.push({
        range: { startLineNumber: targetLine, startColumn: targetColumn, endLineNumber: targetLine, endColumn: Math.max(targetColumn + matchLength, targetColumn + 1) },
        options: { isWholeLine: false, className: "search-jump-match-decoration" },
      });
    }
    decorationsRef.current = editor.deltaDecorations(decorationsRef.current, decorations);
  };

  useEffect(() => {
    jumpToSearchLine();
  }, [searchJumpRequest, searchLine, searchColumn, searchMatchLength, searchTargetPath, selectedBlob?.path, selectedBlob?.content]);

  useEffect(() => {
    if (monacoRef.current) registerEditorTheme(monacoRef.current);
  }, [editorTheme]);

  useEffect(() => () => {
    if (editorRef.current && decorationsRef.current.length > 0) {
      decorationsRef.current = editorRef.current.deltaDecorations(decorationsRef.current, []);
    }
  }, []);

  return (
    <main className="min-w-0 bg-background xl:border-r xl:border-border">
      <div className="flex min-h-11 items-center justify-between gap-3 border-b border-border px-4">
        <div className="flex min-w-0 items-center gap-2">
          {selectedBlob ? <FileCode2 className="size-3.5 shrink-0 text-muted-foreground" /> : <BookOpen className="size-3.5 shrink-0 text-muted-foreground" />}
          <nav aria-label={t("File path")} className="flex min-w-0 items-center gap-1 overflow-hidden font-mono text-xs">
            <span className="shrink-0 text-muted-foreground">{repository.name}</span>
            {(selectedBlob?.path ?? readmePath ?? t("README Preview")).split("/").map((part, index, parts) => (
              <span key={`${part}-${index}`} className="flex min-w-0 items-center gap-1">
                <span aria-hidden="true" className="text-muted-foreground">/</span>
                <span className={index === parts.length - 1 ? "truncate font-medium text-foreground" : "truncate text-muted-foreground"}>{part}</span>
              </span>
            ))}
          </nav>
        </div>
        {selectedBlob ? <span className="shrink-0 font-mono text-[10px] text-muted-foreground">{selectedBlob.size} B</span> : null}
      </div>
      {selectedBlob ? (
        selectedBlob.is_binary ? (
          <div className="flex h-[68vh] min-h-96 items-center justify-center p-6 text-sm text-muted-foreground">
            {t("This file is binary cannot be shown in Monaco.")}
          </div>
        ) : (
          <Editor
            height="68vh"
            language={detectLanguage(selectedBlob.path)}
            value={selectedBlob.content}
            theme={editorTheme}
            onMount={(instance, monaco) => {
              editorRef.current = instance;
              monacoRef.current = monaco;
              registerEditorTheme(monaco);
              jumpToSearchLine();
            }}
            options={{ readOnly: true, minimap: { enabled: false }, fontSize: 13, wordWrap: "on", scrollBeyondLastLine: false }}
          />
        )
      ) : isEmptyRepository ? (
        <RepositoryEmptyState
          t={t}
          repositoryName={repository.name}
          cloneHttpUrl={repository.clone_http_url}
          defaultBranch={repository.default_branch || "main"}
        />
      ) : readmePreview ? (
        <div className="h-[68vh] min-h-96 overflow-auto p-5">
          <div className="mb-4 flex items-center justify-between gap-3">
            <p className="text-xs font-medium text-muted-foreground">{t("README Preview")}</p>
            {readmePath ? <Button type="button" size="sm" variant="outline" onClick={() => onOpenFile(readmePath)}>{t("View README source")}</Button> : null}
          </div>
          <article className="markdown-body" dangerouslySetInnerHTML={{ __html: readmePreview }} />
        </div>
      ) : (
        <div className="flex h-[68vh] min-h-96 items-center justify-center p-6 text-center text-sm text-muted-foreground">
          {t("README not found. Select a source file from the left to open in Monaco.")}
        </div>
      )}
    </main>
  );
};
