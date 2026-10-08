import { Link } from "react-router";
import { ArrowLeft, Copy, GitBranch, ShieldCheck } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { RepositoryView } from "@/pages/types";
import type { RepositoryPermissions } from "./repository-permissions";

interface RepositoryHeaderCardProps {
  organizationName?: string;
  repository: RepositoryView | null;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onCopyCloneUrl: () => void;
}

export const RepositoryHeaderCard = ({
  organizationName, repository, permissions, t, onCopyCloneUrl,
}: RepositoryHeaderCardProps): JSX.Element => {
  const projectMark = (repository?.key || repository?.name || "PR").slice(0, 2).toUpperCase();
  const namespacePath = repository?.full_path
    ?? `${organizationName ?? t("Organization")} / ${repository?.key ?? t("Project")}`;

  return (
    <section
      aria-labelledby="repository-title"
      className="overflow-hidden rounded-xl border border-border bg-card shadow-sm card-enter"
    >
      <div className="flex flex-col gap-4 px-4 py-4 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex min-w-0 items-start gap-3">
          <div className="flex size-10 shrink-0 items-center justify-center rounded-lg border border-border bg-muted font-mono text-sm font-semibold text-foreground">
            {projectMark}
          </div>
          <div className="min-w-0">
            <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
              <h1 id="repository-title" className="truncate text-lg font-semibold tracking-tight">
                {repository?.name ?? t("Project")}
              </h1>
              <Badge variant="secondary" className="font-mono text-[10px] uppercase tracking-wide">
                {repository?.visibility ?? t("N/A")}
              </Badge>
              {repository?.status && repository.status !== "active" ? (
                <Badge variant="outline">{repository.status}</Badge>
              ) : null}
            </div>
            <p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">{namespacePath}</p>
            <p className="mt-1 line-clamp-1 max-w-3xl text-xs text-muted-foreground">
              {repository?.description || t("No description provided.")}
            </p>
          </div>
        </div>

        <div className="flex shrink-0 flex-wrap items-center gap-2">
          <Badge variant="outline" className="h-8 gap-1.5 font-normal">
            <GitBranch className="size-3.5" />
            <span className="font-mono">{repository?.default_branch ?? "main"}</span>
          </Badge>
          <Badge variant="outline" className="h-8 gap-1.5 font-normal">
            <ShieldCheck className="size-3.5" />
            {t(permissions.roleLabel)}
          </Badge>
          <Button type="button" variant="outline" size="sm" asChild>
            <Link to="/app/projects">
              <ArrowLeft data-icon="inline-start" />
              {t("Projects")}
            </Link>
          </Button>
          <Button type="button" size="sm" onClick={onCopyCloneUrl} disabled={!repository}>
            <Copy data-icon="inline-start" />
            {t("Clone")}
          </Button>
        </div>
      </div>

    </section>
  );
};
