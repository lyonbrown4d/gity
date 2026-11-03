import { CircleDot, Tags, UserRound } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataRow } from "@/components/surface/data-list";
import type { RepositoryIssueView, UserView } from "@/pages/types";
import { formatRelativeTime } from "../issues-utils";
import { formatUserLabel } from "../repository-user-utils";
import { issueAssigneeIDs } from "./model";

export const IssueStat = ({ label, value }: { label: string; value: number }): JSX.Element => (
  <div className="flex items-baseline justify-between gap-2 px-4 py-2">
    <span className="text-xs text-muted-foreground">{label}</span>
    <span className="text-sm font-semibold tabular-nums">{value}</span>
  </div>
);

export const IssueEmptyState = ({
  title, description, actionLabel, onAction,
}: {
  title: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
}): JSX.Element => (
  <div className="px-4 py-10 text-center">
    <p className="text-sm font-medium">{title}</p>
    <p className="mx-auto mt-1 max-w-xl text-sm text-muted-foreground">{description}</p>
    {actionLabel && onAction ? (
      <Button type="button" size="sm" variant="outline" className="mt-3" onClick={onAction}>
        {actionLabel}
      </Button>
    ) : null}
  </div>
);

export const IssueListItem = ({
  issue, userByID, t, onOpen,
}: {
  issue: RepositoryIssueView;
  userByID: Map<string, UserView>;
  t: (text: string) => string;
  onOpen: () => void;
}): JSX.Element => {
  const assigneeIDs = issueAssigneeIDs(issue);
  const labels = issue.labels ?? [];
  return (
    <DataRow className="items-stretch p-0">
      <button type="button" className="flex w-full cursor-pointer items-start gap-4 px-4 py-2.5 text-left" onClick={onOpen}>
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium">
            <span className="mr-1.5 font-mono text-xs text-muted-foreground">#{issue.number}</span>{issue.title}
          </p>
          {issue.description ? <p className="mt-0.5 truncate text-xs text-muted-foreground">{issue.description}</p> : null}
          <p className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
            <span>{t("Opened by")} {formatUserLabel(userByID.get(issue.author_user_id), issue.author_user_id)}</span>
            <span aria-hidden="true">·</span>
            <span>{t("updated")} {formatRelativeTime(issue.updated_at)}</span>
            <span aria-hidden="true">·</span>
            <span className="inline-flex items-center gap-1">
              <UserRound className="size-3" aria-hidden="true" />
              {assigneeIDs.length > 0
                ? assigneeIDs.map((userID) => formatUserLabel(userByID.get(userID), userID)).join(", ")
                : t("Unassigned")}
            </span>
          </p>
        </div>
        <div className="flex shrink-0 flex-col items-end gap-1">
          <Badge variant={issue.status === "open" ? "default" : "secondary"} className="gap-1">
            <CircleDot className="size-3" aria-hidden="true" />{issue.status === "open" ? t("Open") : t("Closed")}
          </Badge>
          <span className="flex flex-wrap justify-end gap-1">
            {labels.length > 0 ? labels.map((label) => (
              <Badge key={`${label.id}:${label.name}`} variant="secondary" className="gap-1">
                <Tags className="size-3" aria-hidden="true" />{label.name}
              </Badge>
            )) : (
              <Badge variant="outline" className="gap-1"><Tags className="size-3" aria-hidden="true" />{t("No labels")}</Badge>
            )}
          </span>
        </div>
      </button>
    </DataRow>
  );
};
