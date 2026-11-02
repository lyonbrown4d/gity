import { ArrowRight, BookOpen, Code2, Copy, GitBranch, GitCommit, ShieldCheck, type LucideIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import type { RepositoryBlobView, RepositoryCommitView, RepositoryView } from "@/pages/types";
import { formatRelativeTime } from "../issues-utils";
import type { RepositoryPermissions } from "../repository-permissions";
import type { RepoTab } from "../repository-types";
import { shortSha } from "../repository-utils";

export interface RepositoryPulseItem {
  label: string;
  value: number;
  description: string;
  actionLabel: string;
  icon: LucideIcon;
  tab: RepoTab;
  isLoading: boolean;
}

interface RepositoryOverviewLayoutProps {
  repository: RepositoryView;
  permissions: RepositoryPermissions;
  readmeBlob: RepositoryBlobView | null;
  readmeHtml: string | null;
  isLoadingReadme: boolean;
  recentCommits: RepositoryCommitView[];
  latestCommit: RepositoryCommitView | null;
  isLoadingCommits: boolean;
  protectedBranches: number;
  pulseItems: RepositoryPulseItem[];
  t: (text: string) => string;
  onOpenTab: (tab: RepoTab) => void;
  onCopyCloneUrl: () => void;
}

export const RepositoryOverviewLayout = ({
  repository, permissions, readmeBlob, readmeHtml, isLoadingReadme,
  recentCommits, latestCommit, isLoadingCommits, protectedBranches,
  pulseItems, t, onOpenTab, onCopyCloneUrl,
}: RepositoryOverviewLayoutProps): JSX.Element => (
  <section className="overflow-hidden rounded-xl border border-border bg-card shadow-sm card-enter">
    <div className="flex flex-col gap-3 px-4 py-3 md:flex-row md:items-center md:justify-between">
      <div className="min-w-0">
        <h2 className="text-sm font-semibold">{t("Repository workspace")}</h2>
        <p className="truncate font-mono text-[11px] text-muted-foreground">{repository.clone_http_url}</p>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <Button type="button" variant="outline" size="sm" onClick={onCopyCloneUrl}>
          <Copy data-icon="inline-start" />
          {t("Copy clone URL")}
        </Button>
        <Button type="button" size="sm" onClick={() => onOpenTab("code")}>
          <Code2 data-icon="inline-start" />
          {t("Browse code")}
        </Button>
      </div>
    </div>

    <dl className="grid border-y border-border sm:grid-cols-2 xl:grid-cols-4">
      <Fact
        icon={GitBranch}
        label={t("Default branch")}
        value={repository.default_branch || "main"}
        detail={String(protectedBranches) + " " + t("protected")}
      />
      <Fact
        icon={ShieldCheck}
        label={t("Access role")}
        value={t(permissions.roleLabel)}
        detail={permissions.canWrite ? t("Write access") : t("Read-only access")}
      />
      <Fact
        icon={GitCommit}
        label={t("Latest commit")}
        value={latestCommit ? shortSha(latestCommit.commit_sha) : t("None")}
        detail={latestCommit ? formatRelativeTime(latestCommit.created_at) : t("No commits yet")}
      />
      <Fact
        icon={Code2}
        label={t("Visibility")}
        value={repository.visibility || t("N/A")}
        detail={repository.status ?? t("N/A")}
      />
    </dl>

    <div className="grid xl:grid-cols-[minmax(0,1fr)_360px]">
      <main className="min-w-0 xl:border-r xl:border-border">
        <div className="flex h-12 items-center justify-between border-b border-border px-4">
          <div className="flex min-w-0 items-center gap-2">
            <BookOpen className="size-4 shrink-0 text-muted-foreground" />
            <div className="min-w-0">
              <h3 className="text-sm font-medium">{t("README")}</h3>
              <p className="truncate font-mono text-[10px] text-muted-foreground">{readmeBlob?.path ?? t("Quick entry to project documentation.")}</p>
            </div>
          </div>
          <Button type="button" variant="ghost" size="sm" onClick={() => onOpenTab("code")}>
            {readmeBlob ? t("View source") : t("Open code")}
            <ArrowRight data-icon="inline-end" />
          </Button>
        </div>
        <div className="min-h-[32rem] p-5">
          {isLoadingReadme ? (
            <div className="space-y-3">
              <Skeleton className="h-7 w-2/3" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-5/6" />
              <Skeleton className="h-40 w-full" />
            </div>
          ) : readmeBlob?.is_binary ? (
            <p className="text-sm text-muted-foreground">{t("README is binary and cannot be rendered here. Open the code tab to inspect it.")}</p>
          ) : readmeHtml ? (
            <article className="markdown-body max-h-[54rem] overflow-auto" dangerouslySetInnerHTML={{ __html: readmeHtml }} />
          ) : (
            <div className="flex min-h-72 flex-col items-center justify-center border border-dashed border-border p-8 text-center">
              <BookOpen className="mb-3 size-8 text-muted-foreground" />
              <p className="text-sm font-medium">{t("No README found")}</p>
              <p className="mt-1 max-w-md text-sm text-muted-foreground">
                {t("Add a README to turn this project home into a useful landing page for contributors.")}
              </p>
              <Button type="button" variant="outline" size="sm" className="mt-4" onClick={() => onOpenTab("code")}>
                {t("Open repository files")}
              </Button>
            </div>
          )}
        </div>
      </main>

      <aside className="min-w-0 bg-muted/10">
        <div className="flex h-12 items-center justify-between border-b border-border px-4">
          <div>
            <h3 className="text-sm font-medium">{t("Repository pulse")}</h3>
            <p className="text-[10px] text-muted-foreground">{t("Current project activity")}</p>
          </div>
          <Badge variant="secondary" className="font-mono text-[10px]">{pulseItems.length}</Badge>
        </div>
        <div>
          {pulseItems.map((item) => (
            <button
              key={item.tab}
              type="button"
              className="flex w-full items-center gap-3 border-b border-border px-4 py-3 text-left hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
              onClick={() => onOpenTab(item.tab)}
              aria-label={item.actionLabel}
            >
              <item.icon className="size-4 shrink-0 text-muted-foreground" />
              <span className="min-w-0 flex-1">
                <span className="block text-xs font-medium">{item.label}</span>
                <span className="block truncate text-[10px] text-muted-foreground">{item.description}</span>
              </span>
              {item.isLoading ? <Skeleton className="h-6 w-8" /> : <span className="font-mono text-lg font-semibold tabular-nums">{item.value}</span>}
            </button>
          ))}
        </div>

        <div className="flex h-12 items-center justify-between border-b border-border px-4">
          <h3 className="text-sm font-medium">{t("Recent commits")}</h3>
          <Button type="button" variant="ghost" size="sm" onClick={() => onOpenTab("commits")}>{t("View all")}</Button>
        </div>
        <div>
          {isLoadingCommits ? (
            <div className="space-y-2 p-4"><Skeleton className="h-12 w-full" /><Skeleton className="h-12 w-full" /></div>
          ) : recentCommits.length > 0 ? recentCommits.map((commit) => (
            <button
              key={commit.branch_name + commit.commit_sha}
              type="button"
              className="w-full border-b border-border px-4 py-3 text-left hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
              onClick={() => onOpenTab("commits")}
            >
              <span className="flex items-center gap-2">
                <Badge variant="outline" className="font-mono text-[10px]">{shortSha(commit.commit_sha)}</Badge>
                <span className="truncate font-mono text-[10px] text-muted-foreground">{commit.branch_name}</span>
              </span>
              <span className="mt-1 block truncate text-xs font-medium">{firstLine(commit.message) || t("No commit message")}</span>
              <span className="mt-0.5 block text-[10px] text-muted-foreground">{commit.author_user_id} · {formatRelativeTime(commit.created_at)}</span>
            </button>
          )) : <p className="p-4 text-xs text-muted-foreground">{t("No commits found.")}</p>}
        </div>
      </aside>
    </div>
  </section>
);

const Fact = ({ icon: Icon, label, value, detail }: { icon: LucideIcon; label: string; value: string; detail: string }): JSX.Element => (
  <div className="flex min-w-0 items-center gap-3 border-b border-border px-4 py-3 last:border-b-0 sm:border-b-0 sm:border-r sm:last:border-r-0">
    <Icon className="size-4 shrink-0 text-muted-foreground" />
    <div className="min-w-0">
      <dt className="text-[10px] uppercase tracking-wider text-muted-foreground">{label}</dt>
      <dd className="flex min-w-0 items-baseline gap-2">
        <span className="truncate font-mono text-xs font-semibold">{value}</span>
        <span className="truncate text-[10px] text-muted-foreground">{detail}</span>
      </dd>
    </div>
  </div>
);

const firstLine = (value: string): string => value.split(/\r?\n/g)[0]?.trim() ?? "";
