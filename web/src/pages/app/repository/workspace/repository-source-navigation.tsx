import { useId } from "react";
import { ChevronDown, ChevronRight, FileCode2, Files, FolderTree, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { RepositorySearchResultView } from "@/pages/types";
import type { RepositoryTreeNode } from "../repository-types";

interface RepositorySourceNavigationProps {
  t: (text: string) => string;
  treeNodes: RepositoryTreeNode[];
  selectedPath?: string;
  isLoadingTree: boolean;
  searchQuery: string;
  searchPath: string;
  searchMatchCase: boolean;
  searchRegex: boolean;
  isLoadingSearch: boolean;
  searchResults: RepositorySearchResultView[];
  onOpenFile: (path: string) => void;
  onToggleTreeDirectory: (path: string) => void;
  onOpenFileAtLine: (path: string, line: number, column: number, matchLength?: number) => void;
  onSearch: () => void;
  setSearchQuery: (value: string) => void;
  setSearchPath: (value: string) => void;
  setSearchMatchCase: (value: boolean) => void;
  setSearchRegex: (value: boolean) => void;
}

export const RepositorySourceNavigation = ({
  t, treeNodes, selectedPath, isLoadingTree, searchQuery, searchPath,
  searchMatchCase, searchRegex, isLoadingSearch, searchResults, onOpenFile,
  onToggleTreeDirectory, onOpenFileAtLine, onSearch, setSearchQuery,
  setSearchPath, setSearchMatchCase, setSearchRegex,
}: RepositorySourceNavigationProps): JSX.Element => (
  <aside className="min-w-0 bg-muted/20 xl:border-r xl:border-border">
    <div className="flex h-10 items-center justify-between border-b border-border px-3">
      <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        <Files className="size-3.5" />
        {t("Project files")}
      </div>
      <span className="font-mono text-[10px] text-muted-foreground">{treeNodes.length}</span>
    </div>

    <div className="space-y-2 border-b border-border p-2">
      <div className="relative">
        <Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
        <Input
          className="h-8 pl-8 text-xs"
          aria-label={t("Search text")}
          placeholder={t("Search in project")}
          value={searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
          onKeyDown={(event) => event.key === "Enter" && onSearch()}
        />
      </div>
      <Input
        className="h-8 text-xs"
        aria-label={t("Path (optional)")}
        placeholder={t("Path (optional)")}
        value={searchPath}
        onChange={(event) => setSearchPath(event.target.value)}
        onKeyDown={(event) => event.key === "Enter" && onSearch()}
      />
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 px-1">
        <SearchOption label={t("Match case")} checked={searchMatchCase} onChange={setSearchMatchCase} />
        <SearchOption label={t("Regular expression")} checked={searchRegex} onChange={setSearchRegex} />
        <Button type="button" size="sm" variant="secondary" className="ml-auto h-7 px-2 text-xs" onClick={onSearch} disabled={isLoadingSearch}>
          {isLoadingSearch ? t("Searching...") : t("Search")}
        </Button>
      </div>
    </div>

    {searchResults.length > 0 || (searchQuery.trim() && !isLoadingSearch) ? (
      <div className="border-b border-border">
        <p className="px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
          {searchResults.length > 0 ? `${searchResults.length} ${t("result(s)")}` : t("No matches found.")}
        </p>
        {searchResults.length > 0 ? (
          <ScrollArea className="h-44">
            <div className="pb-2">
              {searchResults.map((result, index) => (
                <button
                  key={`${result.path}-${result.line_number}-${result.column}-${result.match_length}-${index}`}
                  type="button"
                  className="w-full border-t border-border/60 px-3 py-2 text-left hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
                  onClick={() => onOpenFileAtLine(result.path, result.line_number, result.column, result.match_length)}
                >
                  <span className="flex items-center justify-between gap-2 font-mono text-[11px]">
                    <span className="truncate">{result.path}</span>
                    <span className="shrink-0 text-muted-foreground">{result.line_number}:{result.column}</span>
                  </span>
                  <span className="mt-1 block truncate text-[11px] text-muted-foreground">{result.line_content || t("No content")}</span>
                </button>
              ))}
            </div>
          </ScrollArea>
        ) : null}
      </div>
    ) : null}

    <ScrollArea className="h-[52vh] min-h-80">
      <div className="py-1">
        {isLoadingTree ? <p className="px-3 py-2 text-xs text-muted-foreground">{t("Loading files...")}</p> : null}
        {!isLoadingTree ? (
          <TreeNodes nodes={treeNodes} selectedPath={selectedPath} t={t} onOpenFile={onOpenFile} onToggle={onToggleTreeDirectory} />
        ) : null}
        {!isLoadingTree && treeNodes.length === 0 ? <p className="px-3 py-2 text-xs text-muted-foreground">{t("No files found.")}</p> : null}
      </div>
    </ScrollArea>
  </aside>
);

interface TreeNodesProps {
  nodes: RepositoryTreeNode[];
  selectedPath?: string;
  depth?: number;
  t: (text: string) => string;
  onOpenFile: (path: string) => void;
  onToggle: (path: string) => void;
}

const TreeNodes = ({ nodes, selectedPath, depth = 0, t, onOpenFile, onToggle }: TreeNodesProps): JSX.Element => (
  <>
    {nodes.map((node) => (
      <div key={node.path}>
        <button
          type="button"
          aria-current={selectedPath === node.path ? "page" : undefined}
          className="flex h-7 w-full items-center gap-1.5 truncate px-2 text-left font-mono text-xs text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring aria-[current=page]:bg-muted aria-[current=page]:text-foreground"
          style={{ paddingLeft: `${depth * 14 + 8}px` }}
          onClick={() => node.kind === "tree" ? onToggle(node.path) : onOpenFile(node.path)}
        >
          {node.kind === "tree" ? (node.expanded ? <ChevronDown className="size-3.5 shrink-0" /> : <ChevronRight className="size-3.5 shrink-0" />) : <span className="size-3.5 shrink-0" />}
          {node.kind === "tree" ? <FolderTree className="size-3.5 shrink-0" /> : <FileCode2 className="size-3.5 shrink-0" />}
          <span className="truncate">{node.name}</span>
        </button>
        {node.kind === "tree" && node.expanded ? (
          node.loading ? <p className="py-1 text-xs text-muted-foreground" style={{ paddingLeft: `${(depth + 1) * 14 + 8}px` }}>{t("Loading files...")}</p>
            : <TreeNodes nodes={node.children} selectedPath={selectedPath} depth={depth + 1} t={t} onOpenFile={onOpenFile} onToggle={onToggle} />
        ) : null}
      </div>
    ))}
  </>
);

const SearchOption = ({ label, checked, onChange }: { label: string; checked: boolean; onChange: (value: boolean) => void }): JSX.Element => {
  const id = useId();

  return (
    <div className="inline-flex items-center gap-1.5">
      <Checkbox id={id} checked={checked} onCheckedChange={(value) => onChange(value === true)} />
      <label htmlFor={id} className="cursor-pointer text-[11px] text-muted-foreground">
        {label}
      </label>
    </div>
  );
};
