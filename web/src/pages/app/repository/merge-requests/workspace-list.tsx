import type { FormEvent } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { DataList } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type { RepositoryBranchView, RepositoryMergeRequestCheckStatusView, RepositoryMergeRequestView } from "@/pages/types";
import { BranchSelect } from "./branch-select";
import { MergeRequestEmptyState, MergeRequestListItem } from "./list-summary";
import { mergeRequestBranchFilterValue, type MergeRequestBranchFilter, type MergeRequestBranchFilterOption, type MergeRequestStateFilter } from "./filters";

interface MergeRequestListWorkspaceProps {
  t: (text: string) => string;
  branches: RepositoryBranchView[];
  isComposerOpen: boolean;
  canCreate: boolean;
  isCreating: boolean;
  onToggleComposer: () => void;
  onCreate: (event: FormEvent<HTMLFormElement>) => void;
  title: string;
  setTitle: (value: string) => void;
  description: string;
  setDescription: (value: string) => void;
  sourceBranch: string;
  setSourceBranch: (value: string) => void;
  targetBranch: string;
  setTargetBranch: (value: string) => void;
  stateFilter: MergeRequestStateFilter;
  setStateFilter: (value: MergeRequestStateFilter) => void;
  sourceBranchFilter: MergeRequestBranchFilter;
  setSourceBranchFilter: (value: MergeRequestBranchFilter) => void;
  targetBranchFilter: MergeRequestBranchFilter;
  setTargetBranchFilter: (value: MergeRequestBranchFilter) => void;
  sourceBranchOptions: MergeRequestBranchFilterOption[];
  targetBranchOptions: MergeRequestBranchFilterOption[];
  searchQuery: string;
  setSearchQuery: (value: string) => void;
  hasFilters: boolean;
  onResetFilters: () => void;
  total: number;
  isLoading: boolean;
  mergeRequests: RepositoryMergeRequestView[];
  filteredMergeRequests: RepositoryMergeRequestView[];
  selectedMergeRequest: RepositoryMergeRequestView | null;
  selectedChecks: RepositoryMergeRequestCheckStatusView | null;
  approvalsCount: number;
  isLoadingChecks: boolean;
  onSelect: (iid: number) => void;
}

export const MergeRequestListWorkspace = (props: MergeRequestListWorkspaceProps): JSX.Element => {
  const {
    t, branches, isComposerOpen, canCreate, isCreating, onToggleComposer, onCreate,
    title, setTitle, description, setDescription, sourceBranch, setSourceBranch,
    targetBranch, setTargetBranch, stateFilter, setStateFilter, sourceBranchFilter,
    setSourceBranchFilter, targetBranchFilter, setTargetBranchFilter,
    sourceBranchOptions, targetBranchOptions, searchQuery, setSearchQuery,
    hasFilters, onResetFilters, total, isLoading, mergeRequests, filteredMergeRequests,
    selectedMergeRequest, selectedChecks, approvalsCount, isLoadingChecks, onSelect,
  } = props;
  return (
    <Section flush className="overflow-hidden">
      <div className="border-b border-border px-4 py-2.5">
        <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-1">
          <Input
            aria-label={t("Search title, branch, or description")}
            placeholder={t("Search title, branch, or description")}
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
          />
          <Select value={stateFilter} onValueChange={(value) => setStateFilter(value as MergeRequestStateFilter)}>
            <SelectTrigger><SelectValue placeholder={t("Status")} /></SelectTrigger>
            <SelectContent><SelectGroup>
              <SelectItem value="opened">{t("Open merge requests")}</SelectItem>
              <SelectItem value="merged">{t("Merged merge requests")}</SelectItem>
              <SelectItem value="closed">{t("Closed merge requests")}</SelectItem>
              <SelectItem value="all">{t("All merge requests")}</SelectItem>
            </SelectGroup></SelectContent>
          </Select>
          <BranchFilterSelect t={t} label={t("Source branch")} value={sourceBranchFilter} options={sourceBranchOptions} onChange={setSourceBranchFilter} />
          <BranchFilterSelect t={t} label={t("Target branch")} value={targetBranchFilter} options={targetBranchOptions} onChange={setTargetBranchFilter} />
        </div>
        <div className="mt-3 flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
          <span>{t("Showing")} {filteredMergeRequests.length} {t("of")} {total} {t("merge requests")}</span>
          {hasFilters ? <Button type="button" size="sm" variant="ghost" onClick={onResetFilters}>{t("Clear filters")}</Button> : null}
        </div>
      </div>
      {!canCreate ? <div className="border-b border-border px-4 py-2.5"><Alert><AlertDescription>{t("Your current project role can inspect merge requests, but cannot create them.")}</AlertDescription></Alert></div> : null}
      {isComposerOpen ? (
        <form className="flex flex-col gap-2 border-b border-border px-4 py-2.5" onSubmit={onCreate}>
          <Input placeholder={t("Merge request title")} value={title} onChange={(event) => setTitle(event.target.value)} required />
          <Textarea className="min-h-24" placeholder={t("Describe the merge request (optional)")} value={description} onChange={(event) => setDescription(event.target.value)} />
          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-1">
            <BranchSelect id="source-branch" label={t("Source branch")} value={sourceBranch} branches={branches} placeholder={t("Select source branch")} onChange={setSourceBranch} />
            <BranchSelect id="target-branch" label={t("Target branch")} value={targetBranch} branches={branches} placeholder={t("Select target branch")} onChange={setTargetBranch} />
          </div>
          <div className="flex justify-end"><Button type="submit" size="sm" disabled={!canCreate || isCreating}>{isCreating ? t("Creating merge request...") : t("Create merge request")}</Button></div>
        </form>
      ) : null}
      <div>
        {isLoading ? <p className="px-4 py-3 text-sm text-muted-foreground">{t("Loading merge requests...")}</p> : null}
        {!isLoading && mergeRequests.length === 0 ? (
          <MergeRequestEmptyState title={t("No merge requests yet.")} description={canCreate ? t("Create a merge request to start branch review and merge triage.") : t("Merge requests will appear here after they are created.")} actionLabel={canCreate ? t("Create merge request") : undefined} onAction={canCreate ? onToggleComposer : undefined} />
        ) : null}
        {!isLoading && mergeRequests.length > 0 && filteredMergeRequests.length === 0 ? (
          <MergeRequestEmptyState title={t("No merge requests match these filters.")} description={t("Try another state, source branch, target branch, or search term.")} actionLabel={hasFilters ? t("Clear filters") : undefined} onAction={hasFilters ? onResetFilters : undefined} />
        ) : null}
        <DataList>
          {filteredMergeRequests.map((mergeRequest) => {
            const isSelected = selectedMergeRequest?.iid === mergeRequest.iid;
            return <MergeRequestListItem key={mergeRequest.id || mergeRequest.iid} mergeRequest={mergeRequest} isSelected={isSelected} checks={isSelected ? selectedChecks : null} approvalsCount={isSelected ? approvalsCount : null} isLoadingChecks={isSelected && isLoadingChecks} t={t} onSelect={() => onSelect(mergeRequest.iid)} />;
          })}
        </DataList>
      </div>
    </Section>
  );
};

const BranchFilterSelect = ({ t, label, value, options, onChange }: {
  t: (text: string) => string;
  label: string;
  value: MergeRequestBranchFilter;
  options: MergeRequestBranchFilterOption[];
  onChange: (value: MergeRequestBranchFilter) => void;
}): JSX.Element => (
  <Select value={value} onValueChange={(next) => onChange(next as MergeRequestBranchFilter)}>
    <SelectTrigger><SelectValue placeholder={label} /></SelectTrigger>
    <SelectContent><SelectGroup>
      <SelectItem value="all">{t(`Any ${label.toLowerCase()}`)}</SelectItem>
      {options.map((branch) => <SelectItem key={branch.name} value={mergeRequestBranchFilterValue(branch.name)}>{branch.name} ({branch.count})</SelectItem>)}
    </SelectGroup></SelectContent>
  </Select>
);
