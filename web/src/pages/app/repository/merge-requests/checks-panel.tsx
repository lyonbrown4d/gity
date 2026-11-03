import { Clock3, RefreshCw } from "lucide-react";
import { Link } from "react-router";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/surface/section";
import type { RepositoryMergeRequestCheckStatusView } from "@/pages/types";
import { CheckLine, checkStatusMeta, shortText } from "./status";

export const MergeRequestChecksPanel = ({
  organizationId,
  repoId,
  checks,
  isLoading,
  t,
  onReload,
}: {
  organizationId: string;
  repoId: string;
  checks: RepositoryMergeRequestCheckStatusView | null;
  isLoading: boolean;
  t: (text: string) => string;
  onReload: () => void;
}) => {
  const getRepoTabPath = (tab: string) => (
    `/app/projects/${encodeURIComponent(organizationId)}/${encodeURIComponent(repoId)}/${encodeURIComponent(tab)}`
  );
  const renderBlockerHint = (code: string, message: string) => {
    const commonDetails = [message];
    switch (code) {
      case "approval_rule_unsatisfied":
        return {
          title: t("Approval requirements are not met"),
          details: [
            ...commonDetails,
            t("Have a required reviewer approve this merge request."),
            t("Or adjust approval rules for the target branch."),
          ],
          action: {
            label: t("Review approval rules"),
            href: `/app/projects/${encodeURIComponent(organizationId)}/${encodeURIComponent(repoId)}/merge-requests#merge-request-approval-rules-panel`,
          },
        };
      case "pipeline_repository_missing":
        return {
          title: t("Pipeline service is not configured"),
          details: [
            ...commonDetails,
            t("Enable/attach CI runtime for this repository."),
            t("Create a CI config file in the source branch to trigger pipeline generation."),
          ],
          action: {
            label: t("Open code to add CI config"),
            href: getRepoTabPath("code"),
          },
        };
      case "pipeline_missing":
        return {
          title: t("Pipeline not found"),
          details: [
            ...commonDetails,
            t("Push a new commit or fix CI config to trigger a new pipeline."),
            t("Check the pipeline tab for queued or recent pipeline runs."),
          ],
          action: {
            label: t("Open pipelines"),
            href: getRepoTabPath("pipelines"),
          },
        };
      case "pipeline_not_succeeded":
        return {
          title: t("Pipeline did not succeed"),
          details: [
            ...commonDetails,
            t("Fix pipeline failures and rerun the failed pipeline."),
            t("Open pipeline logs from the Pipelines tab."),
          ],
          action: {
            label: t("Open pipelines"),
            href: getRepoTabPath("pipelines"),
          },
        };
      default:
        return {
          title: t("Policy check blocked"),
          details: commonDetails,
        };
    }
  };
  const renderBlockerAction = (href: string, label: string) => {
    if (href.startsWith("#")) {
      return (
        <a href={href} className="text-xs text-primary underline-offset-4 hover:underline">
          {label}
        </a>
      );
    }
    return (
      <Link to={href} className="text-xs text-primary underline-offset-4 hover:underline">
        {label}
      </Link>
    );
  };
  const meta = checks ? checkStatusMeta(checks) : { label: "Not loaded", Icon: Clock3 };
  const Icon = meta.Icon;
  return (
    <Section
      title={
        <span className="flex flex-wrap items-center gap-2">
          <Icon className="size-4" aria-hidden="true" />
          {t("Merge checks")}
          <Badge variant={checks?.mergeable ? "default" : checks?.required ? "outline" : "secondary"}>{t(meta.label)}</Badge>
        </span>
      }
      description={checks?.blocking_reason ? checks.blocking_reason : t("Branch policy and pipeline state for this merge request.")}
      actions={
        <Button type="button" size="sm" variant="ghost" onClick={onReload} disabled={isLoading}>
          <RefreshCw data-icon="inline-start" />
          {isLoading ? t("Checking...") : t("Reload checks")}
        </Button>
      }
      flush
      className="overflow-hidden"
    >
      <div className="flex flex-col gap-2 px-4 py-2.5">
        {checks ? (
          <div className="grid gap-x-4 gap-y-1 text-xs text-muted-foreground grid-cols-1">
            <CheckLine label={t("Target branch")} value={checks.target_branch || t("N/A")} />
            <CheckLine label={t("Source commit")} value={checks.source_commit_sha ? shortText(checks.source_commit_sha) : t("N/A")} />
            <CheckLine label={t("Protected target")} value={checks.target_branch_protected ? t("Yes") : t("No")} />
            <CheckLine label={t("Pipeline required")} value={checks.pipeline_required ? t("Yes") : t("No")} />
            <CheckLine label={t("Merge request required")} value={checks.require_merge_request ? t("Yes") : t("No")} />
            <CheckLine label={t("Pipeline status")} value={checks.pipeline?.status ? t(checks.pipeline.status) : t(checks.status)} />
          </div>
        ) : null}
        {checks?.blockers?.length ? (
          <div className="border-t border-border pt-2">
            <p className="mb-1.5 text-xs font-medium text-muted-foreground">{t("Blocking reasons")}</p>
            <div className="flex flex-col gap-1.5">
              {checks.blockers.map((blocker) => {
                const blockerHint = renderBlockerHint(blocker.code, blocker.message);
                return (
                  <div
                    key={`${blocker.category}:${blocker.code}:${blocker.message}`}
                    className="flex flex-col gap-0.5 border-l-2 border-destructive/50 pl-2"
                  >
                    <div className="flex flex-wrap items-center gap-2">
                      <Badge variant="outline">{t(blocker.category || "policy")}</Badge>
                      <span className="text-xs font-medium">{blockerHint.title}</span>
                    </div>
                    <ul className="ml-6 list-disc text-muted-foreground text-xs">
                      {blockerHint.details.map((item, index) => (
                        <li key={`${item}:${index}`}>{item}</li>
                      ))}
                    </ul>
                    {blockerHint.action ? (
                      <div className="mt-1">
                        {renderBlockerAction(blockerHint.action.href, blockerHint.action.label)}
                      </div>
                    ) : null}
                  </div>
                );
              })}
            </div>
          </div>
        ) : null}
        {checks?.approval_rules?.length ? (
          <div className="flex flex-col gap-1.5 border-t border-border pt-2">
            <p className="text-xs font-medium text-muted-foreground">
              {t("Approval rules")}: {checks.approval_count}/{checks.required_approvals}
            </p>
            <div className="grid gap-1.5 grid-cols-1">
              {checks.approval_rules.map((rule) => (
                <div key={rule.rule_id || rule.name} className="bg-muted/30 px-2.5 py-1.5 text-xs">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <span className="font-medium">{rule.name}</span>
                    <Badge variant={rule.satisfied ? "default" : "outline"}>
                      {rule.approval_count}/{rule.approvals_required}
                    </Badge>
                  </div>
                  <p className="mt-0.5 text-muted-foreground">
                    {rule.code_owner ? t("CODEOWNERS") : t("Eligible users")} · {rule.target_branch || "*"}
                  </p>
                </div>
              ))}
            </div>
          </div>
        ) : null}
      </div>
    </Section>
  );
};
