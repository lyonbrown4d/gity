import { Code2, FilePlus2, GitBranch } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type {
  RepositoryBlobView,
  RepositoryBranchView,
  RepositoryLanguagesView,
  RepositorySearchResultView,
  RepositoryView,
} from "@/pages/types";
import type { RepositoryTreeNode } from "./repository-types";
import { RepositorySourceCanvas } from "./workspace/repository-source-canvas";
import { RepositorySourceIntelligence } from "./workspace/repository-source-intelligence";
import { RepositorySourceNavigation } from "./workspace/repository-source-navigation";

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
  searchQuery: string;
  searchPath: string;
  searchMatchCase: boolean;
  searchRegex: boolean;
  isLoadingSearch: boolean;
  searchLine: number;
  searchColumn: number;
  searchMatchLength: number;
  searchJumpRequest: number;
  searchTargetPath: string;
  searchResults: RepositorySearchResultView[];
  editorTheme: string;
  onChangeCodeBranch: (branchName: string) => void;
  onOpenCreateFile: () => void;
  onOpenFile: (path: string) => void;
  onToggleTreeDirectory: (path: string) => void;
  onRefreshLanguages: () => void;
  onOpenFileAtLine: (path: string, lineNumber: number, column: number, matchLength?: number) => void;
  onSearch: (query: string) => void;
  setSearchQuery: (value: string) => void;
  setSearchPath: (value: string) => void;
  setSearchMatchCase: (value: boolean) => void;
  setSearchRegex: (value: boolean) => void;
}

export const RepositoryCodeTab = ({
  t, repository, branches, codeBranch, treeNodes, selectedBlob, readmePreview,
  readmePath, languages, isLoadingTree, isLoadingLanguages, searchQuery,
  searchPath, searchMatchCase, searchRegex, isLoadingSearch, searchLine,
  searchColumn, searchMatchLength, searchJumpRequest, searchTargetPath,
  searchResults, editorTheme, onChangeCodeBranch, onOpenCreateFile, onOpenFile,
  onToggleTreeDirectory, onOpenFileAtLine, onSearch, setSearchQuery,
  setSearchPath, setSearchMatchCase, setSearchRegex, onRefreshLanguages,
}: RepositoryCodeTabProps): JSX.Element => {
  const isEmptyRepository = !isLoadingTree && treeNodes.length === 0 && !selectedBlob && !readmePreview;

  return (
    <section className="overflow-hidden rounded-xl border border-border bg-card shadow-sm card-enter">
      <div className="flex flex-col gap-3 px-3 py-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground">
            <Code2 className="size-4" />
          </div>
          <div className="min-w-0">
            <h2 className="text-sm font-semibold">{t("Source workspace")}</h2>
            <p className="truncate font-mono text-[11px] text-muted-foreground">{repository.clone_http_url}</p>
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="outline" className="hidden font-mono text-[10px] sm:inline-flex">
            {treeNodes.length} {t("root item(s)")}
          </Badge>
          <Label htmlFor="code-branch-select" className="sr-only">{t("Branch")}</Label>
          <Select value={codeBranch} onValueChange={onChangeCodeBranch}>
            <SelectTrigger id="code-branch-select" className="h-8 w-44 font-mono text-xs">
              <GitBranch className="size-3.5" />
              <SelectValue placeholder={t("Branch")} />
            </SelectTrigger>
            <SelectContent>
              {branches.map((branch) => <SelectItem key={branch.name} value={branch.name}>{branch.name}</SelectItem>)}
              {branches.length === 0 && repository.default_branch ? (
                <SelectItem value={repository.default_branch}>{repository.default_branch}</SelectItem>
              ) : null}
            </SelectContent>
          </Select>
          <Button type="button" size="sm" onClick={onOpenCreateFile}>
            <FilePlus2 data-icon="inline-start" />
            {t("Create file")}
          </Button>
        </div>
      </div>

      <div className="grid border-t border-border xl:grid-cols-[290px_minmax(0,1fr)_260px]">
        <RepositorySourceNavigation
          t={t}
          treeNodes={treeNodes}
          selectedPath={selectedBlob?.path}
          isLoadingTree={isLoadingTree}
          searchQuery={searchQuery}
          searchPath={searchPath}
          searchMatchCase={searchMatchCase}
          searchRegex={searchRegex}
          isLoadingSearch={isLoadingSearch}
          searchResults={searchResults}
          onOpenFile={onOpenFile}
          onToggleTreeDirectory={onToggleTreeDirectory}
          onOpenFileAtLine={onOpenFileAtLine}
          onSearch={() => onSearch(searchQuery)}
          setSearchQuery={setSearchQuery}
          setSearchPath={setSearchPath}
          setSearchMatchCase={setSearchMatchCase}
          setSearchRegex={setSearchRegex}
        />
        <RepositorySourceCanvas
          t={t}
          repository={repository}
          selectedBlob={selectedBlob}
          readmePreview={readmePreview}
          readmePath={readmePath}
          isEmptyRepository={isEmptyRepository}
          editorTheme={editorTheme}
          searchLine={searchLine}
          searchColumn={searchColumn}
          searchMatchLength={searchMatchLength}
          searchJumpRequest={searchJumpRequest}
          searchTargetPath={searchTargetPath}
          onOpenFile={onOpenFile}
        />
        <RepositorySourceIntelligence
          t={t}
          branch={codeBranch}
          languages={languages}
          isLoading={isLoadingLanguages}
          onRefresh={onRefreshLanguages}
        />
      </div>
    </section>
  );
};
