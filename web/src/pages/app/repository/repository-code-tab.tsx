import Editor from "@monaco-editor/react";
import { ChevronDown, ChevronRight, FileCode2, FolderTree } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import type {
  RepositoryBlobView,
  RepositoryBranchView,
  RepositoryLanguageItemView,
  RepositoryLanguagesView,
  RepositoryView,
} from "@/pages/types";
import type { RepositoryTreeNode } from "./repository-types";
import { detectLanguage, formatBytes, formatTime, languageBarColor } from "./repository-utils";

interface RepositoryCodeTabProps {
  t: (text: string) => string;
  repository: RepositoryView;
  branches: RepositoryBranchView[];
  codeBranch: string;
  treeNodes: RepositoryTreeNode[];
  selectedBlob: RepositoryBlobView | null;
  readmePreview: string | null;
  readmePath: string | null;
  languages: RepositoryLanguagesView | null;
  isLoadingTree: boolean;
  isLoadingLanguages: boolean;
  editorTheme: string;
  onChangeCodeBranch: (branchName: string) => void;
  onOpenCreateFile: () => void;
  onOpenFile: (path: string) => void;
  onToggleTreeDirectory: (path: string) => void;
  onRefreshLanguages: () => void;
}

const renderLanguageRows = (items: RepositoryLanguageItemView[]): JSX.Element[] => {
  return items.map((item) => (
    <div key={item.language} className="space-y-1">
      <div className="flex items-center justify-between gap-2 text-xs">
        <span className="truncate font-medium">{item.language}</span>
        <span className="text-muted-foreground">{item.percentage.toFixed(2)}%</span>
      </div>
      <div className="h-2 overflow-hidden rounded bg-muted">
        <div
          className="h-full"
          style={{
            width: `${Math.max(item.percentage, item.percentage > 0 ? 2 : 0)}%`,
            backgroundColor: languageBarColor(item.language),
          }}
        />
      </div>
    </div>
  ));
};

export const RepositoryCodeTab = ({
  t,
  repository,
  branches,
  codeBranch,
  treeNodes,
  selectedBlob,
  readmePreview,
  readmePath,
  languages,
  isLoadingTree,
  isLoadingLanguages,
  editorTheme,
  onChangeCodeBranch,
  onOpenCreateFile,
  onOpenFile,
  onToggleTreeDirectory,
  onRefreshLanguages,
}: RepositoryCodeTabProps): JSX.Element => {
  const renderTreeNodes = (nodes: RepositoryTreeNode[], depth = 0): JSX.Element[] =>
    nodes.map((node) => (
      <div key={node.path}>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className={`w-full justify-start ${selectedBlob?.path === node.path ? "bg-muted" : ""}`}
          style={{ paddingLeft: `${depth * 14 + 8}px` }}
          onClick={() => {
            if (node.kind === "tree") {
              onToggleTreeDirectory(node.path);
            } else {
              onOpenFile(node.path);
            }
          }}
        >
          {node.kind === "tree" ? (
            node.expanded ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />
          ) : (
            <span className="inline-block size-4" />
          )}
          {node.kind === "tree" ? <FolderTree className="size-4" /> : <FileCode2 className="size-4" />}
          <span className="truncate">{node.name}</span>
        </Button>
        {node.kind === "tree" && node.expanded ? (
          node.loading ? (
            <p className="px-2 py-1 text-xs text-muted-foreground" style={{ paddingLeft: `${(depth + 1) * 14 + 8}px` }}>
              {t("Loading files...")}
            </p>
          ) : (
            renderTreeNodes(node.children, depth + 1)
          )
        ) : null}
      </div>
    ));

  return (
    <Card className="card-enter">
      <CardHeader>
        <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
          <div>
            <CardTitle>{t("Code")}</CardTitle>
            <CardDescription>{t("Repository source view and clone information.")}</CardDescription>
          </div>
          <div className="space-y-2">
            <Label htmlFor="code-branch-select">{t("Branch")}</Label>
            <select
              id="code-branch-select"
              className="h-9 w-full rounded-md border bg-background px-3 text-sm"
              value={codeBranch}
              onChange={(event) => onChangeCodeBranch(event.target.value)}
            >
              {branches.map((branch) => (
                <option key={branch.name} value={branch.name}>
                  {branch.name}
                </option>
              ))}
              {branches.length === 0 && repository.default_branch ? (
                <option value={repository.default_branch}>{repository.default_branch}</option>
              ) : null}
            </select>
            <Button type="button" size="sm" variant="outline" onClick={onOpenCreateFile}>
              {t("Create file and commit")}
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="rounded-md border bg-muted/40 px-3 py-2">
          <p className="text-xs text-muted-foreground">{t("Clone URL")}</p>
          <p className="break-all font-mono text-xs">{repository.clone_http_url}</p>
        </div>

        <div className="grid gap-3 xl:grid-cols-[300px_minmax(0,1fr)_280px]">
          <div className="space-y-2 rounded-md border p-2">
            {isLoadingTree ? <p className="px-2 py-1 text-xs text-muted-foreground">{t("Loading files...")}</p> : null}
            {!isLoadingTree ? renderTreeNodes(treeNodes) : null}
            {!isLoadingTree && treeNodes.length === 0 ? (
              <p className="px-2 py-1 text-xs text-muted-foreground">{t("No files found.")}</p>
            ) : null}
          </div>

          <div className="rounded-md border">
            {selectedBlob ? (
              selectedBlob.is_binary ? (
                <div className="p-4 text-sm text-muted-foreground">
                  {t("This file is binary and cannot be shown in Monaco.")}
                </div>
              ) : (
                <Editor
                  height="68vh"
                  language={detectLanguage(selectedBlob.path)}
                  value={selectedBlob.content}
                  theme={editorTheme}
                  options={{
                    readOnly: true,
                    minimap: { enabled: false },
                    fontSize: 13,
                    wordWrap: "on",
                    scrollBeyondLastLine: false,
                  }}
                />
              )
            ) : readmePreview ? (
              <div className="p-4">
                <div className="mb-3 flex items-center justify-between">
                  <p className="text-xs text-muted-foreground">{t("README Preview")}</p>
                  {readmePath ? (
                    <Button type="button" size="sm" variant="outline" onClick={() => onOpenFile(readmePath)}>
                      {t("View README source")}
                    </Button>
                  ) : null}
                </div>
                <article className="markdown-body" dangerouslySetInnerHTML={{ __html: readmePreview }} />
              </div>
            ) : (
              <div className="p-4 text-sm text-muted-foreground">
                {t("README not found. Select a source file from the left to open in Monaco.")}
              </div>
            )}
          </div>

          <div className="space-y-3 rounded-md border p-3">
            <div className="flex items-center justify-between gap-2">
              <p className="text-sm font-medium">{t("Languages")}</p>
              <Button type="button" size="sm" variant="outline" disabled={!codeBranch || isLoadingLanguages} onClick={onRefreshLanguages}>
                {t("Refresh")}
              </Button>
            </div>

            {isLoadingLanguages ? <p className="text-xs text-muted-foreground">{t("Loading language statistics...")}</p> : null}
            {!isLoadingLanguages && languages?.status === "pending" ? (
              <p className="text-xs text-muted-foreground">{t("Language statistics are being computed...")}</p>
            ) : null}
            {!isLoadingLanguages && languages?.status === "ready" && languages.languages.length > 0 ? (
              <div className="space-y-3">{renderLanguageRows(languages.languages)}</div>
            ) : null}
            {!isLoadingLanguages && (!languages || languages.languages.length === 0) && languages?.status !== "pending" ? (
              <p className="text-xs text-muted-foreground">{t("No language statistics yet.")}</p>
            ) : null}
            {languages?.status === "ready" ? (
              <div className="space-y-1 border-t pt-2 text-xs text-muted-foreground">
                <p>{t("Total size")}: {formatBytes(languages.total_bytes)}</p>
                <p>{t("Last analyzed")}: {languages.analyzed_at ? formatTime(languages.analyzed_at) : t("N/A")}</p>
              </div>
            ) : null}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};
