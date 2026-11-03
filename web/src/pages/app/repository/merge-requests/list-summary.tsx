import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataRow } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { cn } from "@/lib/utils";
import type { RepositoryMergeRequestCheckStatusView, RepositoryMergeRequestView } from "@/pages/types";
import { formatRelativeTime } from "../issues-utils";
import { MergeRequestStateBadge } from "./status";

type BadgeTone = "default" | "secondary" | "outline";
interface MergeRequestListGateSummary {
  gateLabel: string;
  gateVariant: BadgeTone;
  approvalLabel?: string;
  blockerDetail?: string;
}

export const MergeRequestEmptyState = ({
  title,
  description,
  actionLabel,
  onAction,
}: {
  title: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
}) => (
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

export const MergeRequestListItem = ({
  mergeRequest,
  isSelected,
  checks,
  approvalsCount,
  isLoadingChecks,
  t,
  onSelect,
}: {
  mergeRequest: RepositoryMergeRequestView;
  isSelected: boolean;
  checks: RepositoryMergeRequestCheckStatusView | null;
  approvalsCount: number | null;
  isLoadingChecks: boolean;
  t: (text: string) => string;
  onSelect: () => void;
}) => {
  const gateSummary = buildMergeRequestListGateSummary(mergeRequest, checks, approvalsCount, isSelected, isLoadingChecks, t);
  return (
    <DataRow className={cn("items-stretch p-0", isSelected ? "bg-primary/5" : "")}>
      <button
        type="button"
        className="flex w-full cursor-pointer items-start gap-3 px-4 py-2.5 text-left"
        aria-current={isSelected ? "true" : undefined}
        onClick={onSelect}
      >
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <p className="truncate text-sm font-medium">
              <span className="mr-1.5 font-mono text-xs text-muted-foreground">!{mergeRequest.iid}</span>
              {mergeRequest.title}
            </p>
            <MergeRequestStateBadge state={mergeRequest.state} t={t} />
          </div>
          <p className="mt-0.5 text-xs text-muted-foreground">
            {mergeRequest.updated_at ? `${t("updated")} ${formatRelativeTime(mergeRequest.updated_at)}` : t("No update timestamp")}
          </p>
          <div className="mt-1 flex flex-wrap gap-1.5">
            <Badge variant="outline">{t("Source")}: {mergeRequest.source_branch || t("N/A")}</Badge>
            <Badge variant="outline">{t("Target")}: {mergeRequest.target_branch || t("N/A")}</Badge>
            <Badge variant={gateSummary.gateVariant}>{gateSummary.gateLabel}</Badge>
            {gateSummary.approvalLabel ? <Badge variant="secondary">{gateSummary.approvalLabel}</Badge> : null}
          </div>
          {gateSummary.blockerDetail ? (
            <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{gateSummary.blockerDetail}</p>
          ) : null}
        </div>
      </button>
    </DataRow>
  );
};

const buildMergeRequestListGateSummary = (
  mergeRequest: RepositoryMergeRequestView,
  checks: RepositoryMergeRequestCheckStatusView | null,
  approvalsCount: number | null,
  isSelected: boolean,
  isLoadingChecks: boolean,
  t: (text: string) => string,
): MergeRequestListGateSummary => {
  if (mergeRequest.state === "merged") {
    return { gateLabel: t("Merged"), gateVariant: "secondary" };
  }
  if (mergeRequest.state === "closed") {
    return { gateLabel: t("Closed"), gateVariant: "outline" };
  }
  if (!isSelected) {
    return { gateLabel: t("Select to inspect gates"), gateVariant: "secondary" };
  }
  if (isLoadingChecks) {
    return { gateLabel: t("Loading gates"), gateVariant: "secondary" };
  }
  if (!checks) {
    return { gateLabel: t("Gate status unavailable"), gateVariant: "outline" };
  }

  const approvalCount = Math.max(checks.approval_count, approvalsCount ?? 0);
  const approvalLabel = checks.required_approvals > 0
    ? `${t("Approvals")} ${approvalCount}/${checks.required_approvals}`
    : t("No approvals required");
  if (!checks.required) {
    return { gateLabel: t("No blockers"), gateVariant: "secondary", approvalLabel };
  }
  if (checks.mergeable) {
    return { gateLabel: t("Ready to merge"), gateVariant: "default", approvalLabel };
  }

  const firstBlockerMessage = checks.blockers
    .map((blocker) => blocker.message.trim())
    .find((message) => message.length > 0);
  const blockerDetail = firstBlockerMessage
    || checks.blocking_reason
    || (checks.pipeline?.status ? `${t("Pipeline")}: ${t(checks.pipeline.status)}` : t("Review merge checks for details."));
  return {
    gateLabel: t("Blocked"),
    gateVariant: "outline",
    approvalLabel,
    blockerDetail,
  };
};
export const MergeRequestReadinessSummary = ({
  mergeRequest,
  checks,
  approvalsCount,
  reviewersCount,
  assigneesCount,
  commentsCount,
  currentUserApproved,
  t,
}: {
  mergeRequest: RepositoryMergeRequestView;
  checks: RepositoryMergeRequestCheckStatusView | null;
  approvalsCount: number;
  reviewersCount: number;
  assigneesCount: number;
  commentsCount: number;
  currentUserApproved: boolean;
  t: (text: string) => string;
}) => {
  const isOpen = mergeRequest.state === "opened";
  const statusLabel = !isOpen
    ? mergeRequest.state === "merged" ? "Merged" : "Closed"
    : checks?.mergeable ? "Ready to merge" : checks?.required ? "Blocked" : "Review in progress";
  const requiredApprovals = checks?.required_approvals ?? 0;
  const approvalCount = checks?.approval_count ?? approvalsCount;
  return (
    <Section
      title={t("Merge readiness")}
      description={t("GitLab-style merge gate summary for reviewers and maintainers.")}
      actions={<Badge variant={isOpen && checks?.mergeable ? "default" : "secondary"}>{t(statusLabel)}</Badge>}
      flush
      className="overflow-hidden"
    >
      <div className="grid divide-y divide-border sm:grid-cols-2 sm:divide-x sm:divide-y-0 xl:grid-cols-4">
        <MergeRequestSummaryItem label={t("Approvals")} value={`${approvalCount}/${requiredApprovals}`} detail={currentUserApproved ? t("You approved") : t("Your approval pending")} />
        <MergeRequestSummaryItem label={t("Reviewers")} value={String(reviewersCount)} detail={assigneesCount > 0 ? `${assigneesCount} ${t("assignee(s)")}` : t("No assignee")} />
        <MergeRequestSummaryItem label={t("Pipeline")} value={checks?.pipeline?.status ? t(checks.pipeline.status) : t(checks?.status || "unknown")} detail={checks?.pipeline_required ? t("Required") : t("Optional")} />
        <MergeRequestSummaryItem label={t("Discussion")} value={String(commentsCount)} detail={t("comment(s)")} />
      </div>
    </Section>
  );
};

const MergeRequestSummaryItem = ({ label, value, detail }: { label: string; value: string; detail: string }) => (
  <div className="px-4 py-2">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-sm font-semibold tabular-nums">{value}</p>
    <p className="truncate text-xs text-muted-foreground">{detail}</p>
  </div>
);
