import { useMemo, useState } from "react";
import { FilePlus2, GitBranch, ShieldCheck } from "lucide-react";
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
  canWrite: boolean;
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
  t, repository, canWrite, branches, codeBranch, treeNodes, selectedBlob, readmePreview,
  readmePath, languages, isLoadingTree, isLoadingLanguages, searchQuery,
  searchPath, searchMatchCase, searchRegex, isLoadingSearch, searchLine,
  searchColumn, searchMatchLength, searchJumpRequest, searchTargetPath,
  searchResults, editorTheme, onChangeCodeBranch, onOpenCreateFile, onOpenFile,
  onToggleTreeDirectory, onOpenFileAtLine, onSearch, setSearchQuery,
  setSearchPath, setSearchMatchCase, setSearchRegex, onRefreshLanguages,
}: RepositoryCodeTabProps): JSX.Element => {
  const [branchQuery, setBranchQuery] = useState("");
  const filteredBranches = useMemo(() => {
    const query = branchQuery.trim().toLocaleLowerCase();
    return query ? branches.filter((branch) => branch.name.toLocaleLowerCase().includes(query)) : branches;
  }, [branches, branchQuery]);
  const isEmptyRepository = !isLoadingTree && treeNodes.length === 0 && !selectedBlob && !readmePreview;

  return (
    <section className="overflow-hidden rounded-lg border border-border bg-card card-enter">
      <div className="flex flex-col gap-3 border-b border-border px-4 py-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex min-w-0 items-center gap-3">
          <div className="min-w-0">
            <h2 className="text-base font-semibold tracking-tight">{t("Code")}</h2>
            <p className="truncate text-xs text-muted-foreground">{repository.description || t("Browse files on the selected branch.")}</p>
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <div className="flex items-center gap-2">
            <Label htmlFor="code-branch-search" className="sr-only">{t("Search branches")}</Label>
            <input
              id="code-branch-search"
              className="h-8 w-32 rounded-md border border-input bg-background px-2 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring sm:w-40"
              placeholder={t("Search branches")}
              value={branchQuery}
              onChange={(event) => setBranchQuery(event.target.value)}
            />
          </div>
          <Label htmlFor="code-branch-select" className="sr-only">{t("Branch")}</Label>
          <Select value={codeBranch} onValueChange={(value) => { onChangeCodeBranch(value); setBranchQuery(""); }}>
            <SelectTrigger id="code-branch-select" className="h-8 w-44 font-mono text-xs">
              <GitBranch className="size-3.5" />
              <SelectValue placeholder={t("Branch")} />
            </SelectTrigger>
            <SelectContent>
              {filteredBranches.map((branch) => (
                <SelectItem key={branch.name} value={branch.name}>
                  <span className="flex items-center gap-2">
                    <span>{branch.name}</span>
                    {branch.is_protected ? <ShieldCheck className="size-3.5 text-success" aria-label={t("Protected branch")} /> : null}
                  </span>
                </SelectItem>
              ))}
              {branches.length === 0 && repository.default_branch ? (
                <SelectItem value={repository.default_branch}>{repository.default_branch}</SelectItem>
              ) : null}
              {filteredBranches.length === 0 && branches.length > 0 ? <SelectItem value="__no_branch_match__" disabled>{t("No branches match.")}</SelectItem> : null}
            </SelectContent>
          </Select>
          <Button type="button" size="sm" onClick={onOpenCreateFile} disabled={!canWrite} aria-label={!canWrite ? t("Create file requires repository write access") : undefined}>
            <FilePlus2 data-icon="inline-start" />
            {t("Create file")}
          </Button>
        </div>
      </div>

      <div className="grid min-h-[68vh] xl:grid-cols-[270px_minmax(0,1fr)]">
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
      </div>
      <RepositorySourceIntelligence t={t} branch={codeBranch} languages={languages} isLoading={isLoadingLanguages} onRefresh={onRefreshLanguages} />
    </section>
  );
};
