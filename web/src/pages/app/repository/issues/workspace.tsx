import type { FormEvent } from "react";
import { Search } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { DataList } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { IssueMarkdownEditor } from "../issue-markdown-editor";
import type { RepositoryIssueView, UserView } from "@/pages/types";
import type { IssueAssigneeFilter, IssueAssigneeFilterOption, IssueLabelFilter, IssueLabelFilterOption, IssueStatusFilter, RepositoryIssueStats } from "./model";
import type { IssueSortMode } from "../issues-utils";
import { IssueEmptyState, IssueListItem, IssueStat } from "./list-items";
import { formatUserLabel } from "../repository-user-utils";

export interface IssueWorkspaceProps {
  organizationId: string;
  repoId: string;
  t: (text: string) => string;
  onError: (message: string | null) => void;
  issues: RepositoryIssueView[];
  users: UserView[];
  userByID: Map<string, UserView>;
  filteredIssues: RepositoryIssueView[];
  stats: RepositoryIssueStats;
  labelOptions: IssueLabelFilterOption[];
  assigneeOptions: IssueAssigneeFilterOption[];
  isLoading: boolean;
  canCreate: boolean;
  isCreating: boolean;
  isComposerOpen: boolean;
  setComposerOpen: (open: boolean) => void;
  title: string;
  setTitle: (value: string) => void;
  description: string;
  setDescription: (value: string) => void;
  assigneeID: string;
  setAssigneeID: (value: string) => void;
  status: IssueStatusFilter;
  setStatus: (value: IssueStatusFilter) => void;
  searchQuery: string;
  setSearchQuery: (value: string) => void;
  labelFilter: IssueLabelFilter;
  setLabelFilter: (value: IssueLabelFilter) => void;
  assigneeFilter: IssueAssigneeFilter;
  setAssigneeFilter: (value: IssueAssigneeFilter) => void;
  sort: IssueSortMode;
  setSort: (value: IssueSortMode) => void;
  hasFilters: boolean;
  onResetFilters: () => void;
  onReload: () => void;
  onCreate: (event: FormEvent<HTMLFormElement>) => void;
  onOpenIssue: (issue: RepositoryIssueView) => void;
}

export const IssueWorkspace = (props: IssueWorkspaceProps): JSX.Element => {
  const {
    organizationId, repoId, t, onError, issues, users, userByID, filteredIssues, stats,
    labelOptions, assigneeOptions, isLoading, canCreate, isCreating, isComposerOpen,
    setComposerOpen, title, setTitle, description, setDescription, assigneeID,
    setAssigneeID, status, setStatus, searchQuery, setSearchQuery, labelFilter,
    setLabelFilter, assigneeFilter, setAssigneeFilter, sort, setSort, hasFilters,
    onResetFilters, onReload, onCreate, onOpenIssue,
  } = props;
  return (
    <Section title={t("Issues")} description={t("Track bugs, tasks, ownership, labels, and discussions for this project.")}
      actions={<>
        <Button type="button" size="sm" variant={isComposerOpen ? "secondary" : "outline"} disabled={!canCreate} onClick={() => setComposerOpen(!isComposerOpen)}>
          {isComposerOpen ? t("Hide new issue form") : t("New issue")}
        </Button>
        <Button type="button" size="sm" variant="ghost" onClick={onReload}>{t("Reload")}</Button>
      </>} flush className="card-enter overflow-hidden">
      <div className="grid divide-y divide-border border-b border-border sm:grid-cols-2 sm:divide-x sm:divide-y-0 xl:grid-cols-5">
        <IssueStat label={t("Open")} value={stats.open} /><IssueStat label={t("Closed")} value={stats.closed} />
        <IssueStat label={t("Unassigned")} value={stats.unassigned} /><IssueStat label={t("Labeled")} value={stats.labeled} />
        <IssueStat label={t("Total")} value={stats.total} />
      </div>
      <IssueFilters t={t} status={status} setStatus={setStatus} searchQuery={searchQuery} setSearchQuery={setSearchQuery}
        labelFilter={labelFilter} setLabelFilter={setLabelFilter} labelOptions={labelOptions}
        assigneeFilter={assigneeFilter} setAssigneeFilter={setAssigneeFilter} assigneeOptions={assigneeOptions}
        sort={sort} setSort={setSort} hasFilters={hasFilters} onResetFilters={onResetFilters} total={stats.total} count={filteredIssues.length} />
      {!canCreate ? <div className="border-b border-border px-4 py-3"><Alert><AlertDescription>{t("Your current project role can inspect issues, but cannot create them.")}</AlertDescription></Alert></div> : null}
      {isComposerOpen ? <IssueCreateForm organizationId={organizationId} repoId={repoId} t={t} onError={onError}
        users={users} canCreate={canCreate} isCreating={isCreating} title={title} setTitle={setTitle}
        description={description} setDescription={setDescription} assigneeID={assigneeID} setAssigneeID={setAssigneeID}
        onClose={() => setComposerOpen(false)} onSubmit={onCreate} /> : null}
      {isLoading ? <p className="px-4 py-3 text-sm text-muted-foreground">{t("Loading issues...")}</p> : null}
      {!isLoading && issues.length === 0 ? <IssueEmptyState title={t("No issues yet.")}
        description={canCreate ? t("Create the first issue to start triage for this project.") : t("Issues will appear here after they are created.")}
        actionLabel={canCreate ? t("Create an issue") : undefined} onAction={canCreate ? () => setComposerOpen(true) : undefined} /> : null}
      {!isLoading && issues.length > 0 && filteredIssues.length === 0 ? <IssueEmptyState title={t("No issues match these filters.")}
        description={t("Try another state, label, assignee, or search term.")} actionLabel={hasFilters ? t("Clear filters") : undefined}
        onAction={hasFilters ? onResetFilters : undefined} /> : null}
      {filteredIssues.length > 0 ? <DataList>{filteredIssues.map((issue) => <IssueListItem key={issue.id} issue={issue}
        userByID={userByID} t={t} onOpen={() => onOpenIssue(issue)} />)}</DataList> : null}
    </Section>
  );
};

const IssueFilters = ({ t, status, setStatus, searchQuery, setSearchQuery, labelFilter, setLabelFilter, labelOptions,
  assigneeFilter, setAssigneeFilter, assigneeOptions, sort, setSort, hasFilters, onResetFilters, total, count }: {
  t: (text: string) => string; status: IssueStatusFilter; setStatus: (value: IssueStatusFilter) => void;
  searchQuery: string; setSearchQuery: (value: string) => void; labelFilter: IssueLabelFilter;
  setLabelFilter: (value: IssueLabelFilter) => void; labelOptions: IssueLabelFilterOption[];
  assigneeFilter: IssueAssigneeFilter; setAssigneeFilter: (value: IssueAssigneeFilter) => void;
  assigneeOptions: IssueAssigneeFilterOption[]; sort: IssueSortMode; setSort: (value: IssueSortMode) => void;
  hasFilters: boolean; onResetFilters: () => void; total: number; count: number;
}): JSX.Element => (
  <div className="border-b border-border px-4 py-3">
    <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-[minmax(0,1fr)_160px_180px_180px_170px]">
      <div className="relative md:col-span-2 xl:col-span-1"><Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input className="h-8 pl-8" placeholder={t("Search title, label, or assignee")} value={searchQuery} onChange={(event) => setSearchQuery(event.target.value)} />
      </div>
      <Select value={status} onValueChange={(value) => setStatus(value as IssueStatusFilter)}><SelectTrigger className="h-8"><SelectValue placeholder={t("Status")} /></SelectTrigger>
        <SelectContent><SelectGroup><SelectItem value="open">{t("Open issues")}</SelectItem><SelectItem value="closed">{t("Closed issues")}</SelectItem><SelectItem value="all">{t("All issues")}</SelectItem></SelectGroup></SelectContent>
      </Select>
      <Select value={labelFilter} onValueChange={(value) => setLabelFilter(value as IssueLabelFilter)}><SelectTrigger className="h-8"><SelectValue placeholder={t("Label")} /></SelectTrigger>
        <SelectContent><SelectGroup><SelectItem value="all">{t("Any label")}</SelectItem><SelectItem value="unlabeled">{t("Unlabeled")}</SelectItem>
          {labelOptions.map((label) => <SelectItem key={label.name} value={`label:${label.name}`}>{label.name} ({label.count})</SelectItem>)}</SelectGroup></SelectContent>
      </Select>
      <Select value={assigneeFilter} onValueChange={(value) => setAssigneeFilter(value as IssueAssigneeFilter)}><SelectTrigger className="h-8"><SelectValue placeholder={t("Assignee")} /></SelectTrigger>
        <SelectContent><SelectGroup><SelectItem value="all">{t("Any assignee")}</SelectItem><SelectItem value="unassigned">{t("Unassigned")}</SelectItem>
          {assigneeOptions.map((assignee) => <SelectItem key={assignee.id} value={`user:${assignee.id}`}>{assignee.label} ({assignee.count})</SelectItem>)}</SelectGroup></SelectContent>
      </Select>
      <Select value={sort} onValueChange={(value) => setSort(value as IssueSortMode)}><SelectTrigger className="h-8"><SelectValue placeholder={t("Recently updated")} /></SelectTrigger>
        <SelectContent><SelectGroup><SelectItem value="updated_desc">{t("Recently updated")}</SelectItem><SelectItem value="created_desc">{t("Recently created")}</SelectItem>
          <SelectItem value="number_desc">{t("Newest number")}</SelectItem><SelectItem value="number_asc">{t("Oldest number")}</SelectItem></SelectGroup></SelectContent>
      </Select>
    </div>
    <div className="mt-2 flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground"><span>{t("Showing")} {count} {t("of")} {total} {t("issues")}</span>
      {hasFilters ? <Button type="button" size="sm" variant="ghost" onClick={onResetFilters}>{t("Clear filters")}</Button> : null}</div>
  </div>
);

const IssueCreateForm = ({ organizationId, repoId, t, onError, users, canCreate, isCreating, title, setTitle,
  description, setDescription, assigneeID, setAssigneeID, onClose, onSubmit }: {
  organizationId: string; repoId: string; t: (text: string) => string; onError: (message: string | null) => void;
  users: UserView[]; canCreate: boolean; isCreating: boolean; title: string; setTitle: (value: string) => void;
  description: string; setDescription: (value: string) => void; assigneeID: string; setAssigneeID: (value: string) => void;
  onClose: () => void; onSubmit: (event: FormEvent<HTMLFormElement>) => void;
}): JSX.Element => (
  <form className="flex flex-col gap-2 border-b border-border px-4 py-3" onSubmit={onSubmit}>
    <Input className="h-8" placeholder={t("Issue title")} value={title} onChange={(event) => setTitle(event.target.value)} required />
    <IssueMarkdownEditor organizationId={organizationId} repoId={repoId} t={t} value={description} placeholder={t("Describe the issue (optional)")}
      onChange={setDescription} onError={onError} />
    <Select value={assigneeID} onValueChange={setAssigneeID} disabled={users.length === 0}><SelectTrigger className="h-8">
      <SelectValue placeholder={users.length === 0 ? t("No users available") : t("Assign to user (optional)")} /></SelectTrigger>
      <SelectContent><SelectGroup>{users.map((user) => <SelectItem key={user.id} value={user.id}>{formatUserLabel(user, user.id)}</SelectItem>)}</SelectGroup></SelectContent>
    </Select>
    <div className="flex justify-end gap-2"><Button type="button" size="sm" variant="ghost" onClick={onClose}>{t("Cancel")}</Button>
      <Button type="submit" size="sm" disabled={!canCreate || isCreating}>{isCreating ? t("Creating issue...") : t("Create issue")}</Button></div>
  </form>
);
