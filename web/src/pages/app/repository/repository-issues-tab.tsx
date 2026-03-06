import { useEffect, useMemo, useState } from "react";
import { CircleDot, Search } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { apiRequest } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { RepositoryIssueView } from "@/pages/types";
import { buildIssueDetailPath } from "./issue-markdown";
import { IssueMarkdownEditor } from "./issue-markdown-editor";
import { extractErrorMessage, filterAndSortIssues, formatRelativeTime, type IssueSortMode } from "./issues-utils";

interface RepositoryIssuesTabProps {
  organizationId: string;
  repoId: string;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryIssuesTab = ({
  organizationId,
  repoId,
  t,
  onError,
}: RepositoryIssuesTabProps): JSX.Element => {
  const navigate = useNavigate();
  const [issues, setIssues] = useState<RepositoryIssueView[]>([]);
  const [isLoadingIssues, setLoadingIssues] = useState(false);
  const [issueStatusFilter, setIssueStatusFilter] = useState<"open" | "closed" | "all">("open");
  const [issueSearchQuery, setIssueSearchQuery] = useState("");
  const [issueSort, setIssueSort] = useState<IssueSortMode>("updated_desc");
  const [isIssueComposerOpen, setIssueComposerOpen] = useState(false);
  const [newIssueTitle, setNewIssueTitle] = useState("");
  const [newIssueDescription, setNewIssueDescription] = useState("");
  const [newIssueAssigneeId, setNewIssueAssigneeId] = useState("");
  const [isCreatingIssue, setCreatingIssue] = useState(false);

  const issueStats = useMemo(
    () => ({
      total: issues.length,
      open: issues.filter((item) => item.status === "open").length,
      closed: issues.filter((item) => item.status === "closed").length,
    }),
    [issues],
  );
  const filteredIssues = useMemo(
    () => filterAndSortIssues(issues, issueStatusFilter, issueSearchQuery, issueSort),
    [issues, issueStatusFilter, issueSearchQuery, issueSort],
  );

  const loadIssues = async () => {
    setLoadingIssues(true);
    try {
      const query = new URLSearchParams();
      query.set("status", "all");
      query.set("limit", "100");
      const data = await apiRequest<RepositoryIssueView[]>(`/repos/${repoId}/issues?${query.toString()}`);
      setIssues(data);
    } catch (error) {
      onError(extractErrorMessage(error));
    } finally {
      setLoadingIssues(false);
    }
  };

  const submitCreateIssue = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const title = newIssueTitle.trim();
    if (!title) {
      onError(t("Issue title is required"));
      return;
    }

    onError(null);
    setCreatingIssue(true);
    try {
      const issue = await apiRequest<RepositoryIssueView>(`/repos/${repoId}/issues`, {
        method: "POST",
        body: JSON.stringify({
          title,
          description: newIssueDescription.trim() || null,
          assignee_user_id: newIssueAssigneeId.trim() || null,
        }),
      });
      setNewIssueTitle("");
      setNewIssueDescription("");
      setNewIssueAssigneeId("");
      setIssueComposerOpen(false);
      await loadIssues();
      navigate(buildIssueDetailPath(organizationId, repoId, issue.number));
    } catch (error) {
      onError(extractErrorMessage(error));
    } finally {
      setCreatingIssue(false);
    }
  };

  useEffect(() => {
    if (!repoId) {
      return;
    }
    onError(null);
    void loadIssues();
  }, [repoId]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Issues")}</CardTitle>
        <CardDescription>{t("Track bugs, tasks, and discussions for this repository.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <div className="rounded-md border border-emerald-500/30 bg-emerald-500/5 p-3">
            <p className="text-xs text-muted-foreground">{t("Open")}</p>
            <p className="text-lg font-semibold">{issueStats.open}</p>
          </div>
          <div className="rounded-md border border-slate-500/30 bg-slate-500/5 p-3">
            <p className="text-xs text-muted-foreground">{t("Closed")}</p>
            <p className="text-lg font-semibold">{issueStats.closed}</p>
          </div>
          <div className="rounded-md border p-3">
            <p className="text-xs text-muted-foreground">{t("Total")}</p>
            <p className="text-lg font-semibold">{issueStats.total}</p>
          </div>
        </div>

        <div className="rounded-md border p-3">
          <div className="grid gap-2 md:grid-cols-[1fr_auto_auto]">
            <div className="relative">
              <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                className="pl-8"
                placeholder={t("Search issues")}
                value={issueSearchQuery}
                onChange={(event) => setIssueSearchQuery(event.target.value)}
              />
            </div>
            <select
              className="h-9 w-full rounded-md border bg-background px-3 text-sm"
              value={issueStatusFilter}
              onChange={(event) => setIssueStatusFilter(event.target.value as "open" | "closed" | "all")}
            >
              <option value="open">{t("Open issues")}</option>
              <option value="closed">{t("Closed issues")}</option>
              <option value="all">{t("All issues")}</option>
            </select>
            <select
              className="h-9 w-full rounded-md border bg-background px-3 text-sm"
              value={issueSort}
              onChange={(event) => setIssueSort(event.target.value as IssueSortMode)}
            >
              <option value="updated_desc">{t("Recently updated")}</option>
              <option value="created_desc">{t("Recently created")}</option>
              <option value="number_desc">{t("Newest number")}</option>
              <option value="number_asc">{t("Oldest number")}</option>
            </select>
          </div>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isIssueComposerOpen ? "secondary" : "outline"}
            onClick={() => setIssueComposerOpen((current) => !current)}
          >
            {isIssueComposerOpen ? t("Hide new issue form") : t("New issue")}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadIssues()}>
            {t("Reload")}
          </Button>
        </div>

        {isIssueComposerOpen ? (
          <form className="space-y-3 rounded-md border p-3" onSubmit={submitCreateIssue}>
            <Input
              placeholder={t("Issue title")}
              value={newIssueTitle}
              onChange={(event) => setNewIssueTitle(event.target.value)}
              required
            />
            <IssueMarkdownEditor
              organizationId={organizationId}
              repoId={repoId}
              t={t}
              value={newIssueDescription}
              placeholder={t("Describe the issue (optional)")}
              onChange={setNewIssueDescription}
              onError={onError}
            />
            <Input
              placeholder={t("Assignee user ID (optional)")}
              value={newIssueAssigneeId}
              onChange={(event) => setNewIssueAssigneeId(event.target.value)}
            />
            <div className="flex justify-end">
              <Button type="submit" disabled={isCreatingIssue}>
                {isCreatingIssue ? t("Creating issue...") : t("Create issue")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="space-y-2 rounded-md border p-2">
          {isLoadingIssues ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading issues...")}</p> : null}
          {!isLoadingIssues && filteredIssues.length === 0 ? (
            <p className="px-2 py-2 text-sm text-muted-foreground">{t("No issues found.")}</p>
          ) : null}
          {filteredIssues.map((issue) => (
            <button
              key={issue.id}
              type="button"
              className="w-full rounded-md border p-3 text-left transition hover:bg-muted/40"
              onClick={() => navigate(buildIssueDetailPath(organizationId, repoId, issue.number))}
            >
              <div className="flex flex-wrap items-start justify-between gap-2">
                <p className="font-medium">
                  #{issue.number} {issue.title}
                </p>
                <Badge variant={issue.status === "open" ? "default" : "secondary"}>
                  <CircleDot className="mr-1 size-3" />
                  {issue.status === "open" ? t("Open") : t("Closed")}
                </Badge>
              </div>
              {issue.description ? (
                <p className="mt-2 line-clamp-2 text-sm text-muted-foreground">{issue.description}</p>
              ) : null}
              <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                <span>{issue.author_user_id}</span>
                <span>·</span>
                <span>{formatRelativeTime(issue.updated_at)}</span>
              </div>
            </button>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};
