import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import Editor from "@monaco-editor/react";
import DOMPurify from "dompurify";
import { marked } from "marked";
import { FileCode2, FolderTree } from "lucide-react";
import { useDelete, useList } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { apiRequest } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type {
  OrganizationView,
  RepositoryBlobView,
  RepositoryBranchView,
  RepositoryCommitView,
  RepositoryTreeEntryView,
  RepositoryView,
} from "@/pages/types";

type RepoTab = "code" | "commits" | "branches" | "settings";

function shortSha(value: string): string {
  return value.slice(0, 8);
}

function formatTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString();
}

function extractErrorMessage(error: unknown): string {
  if (!(error instanceof Error)) {
    return "Unknown error";
  }
  const raw = error.message.trim();
  if (!raw) {
    return "Unknown error";
  }

  try {
    const parsed = JSON.parse(raw) as { message?: string };
    if (typeof parsed.message === "string" && parsed.message.trim().length > 0) {
      return parsed.message;
    }
  } catch {
    // ignore non-json message
  }

  return raw;
}

function isReadmePath(path: string): boolean {
  return path.split("/").pop()?.toLowerCase().startsWith("readme") ?? false;
}

function detectLanguage(path: string): string {
  const file = path.split("/").pop()?.toLowerCase() ?? "";
  if (file.endsWith(".rs")) {
    return "rust";
  }
  if (file.endsWith(".ts") || file.endsWith(".tsx")) {
    return "typescript";
  }
  if (file.endsWith(".js") || file.endsWith(".jsx")) {
    return "javascript";
  }
  if (file.endsWith(".json")) {
    return "json";
  }
  if (file.endsWith(".md")) {
    return "markdown";
  }
  if (file.endsWith(".toml")) {
    return "ini";
  }
  if (file.endsWith(".yml") || file.endsWith(".yaml")) {
    return "yaml";
  }
  if (file.endsWith(".go")) {
    return "go";
  }
  if (file.endsWith(".java")) {
    return "java";
  }
  if (file.endsWith(".py")) {
    return "python";
  }
  if (file.endsWith(".sh")) {
    return "shell";
  }
  return "plaintext";
}

async function renderMarkdown(content: string): Promise<string> {
  const html = await marked.parse(content);
  return DOMPurify.sanitize(html);
}

export function RepositoryDetailPage(): JSX.Element {
  const { t } = useI18n();
  const navigate = useNavigate();
  const params = useParams<{ organizationId: string; repoId: string }>();
  const organizationId = params.organizationId ?? "";
  const repoId = params.repoId ?? "";

  const [activeTab, setActiveTab] = useState<RepoTab>("code");
  const [branches, setBranches] = useState<RepositoryBranchView[]>([]);
  const [commits, setCommits] = useState<RepositoryCommitView[]>([]);
  const [branchFilter, setBranchFilter] = useState("all");
  const [newBranchName, setNewBranchName] = useState("");
  const [isLoadingBranches, setLoadingBranches] = useState(false);
  const [isLoadingCommits, setLoadingCommits] = useState(false);
  const [isUpdatingBranch, setUpdatingBranch] = useState(false);
  const [isCreatingBranch, setCreatingBranch] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const [codeBranch, setCodeBranch] = useState("");
  const [treePath, setTreePath] = useState("");
  const [treeEntries, setTreeEntries] = useState<RepositoryTreeEntryView[]>([]);
  const [selectedBlob, setSelectedBlob] = useState<RepositoryBlobView | null>(null);
  const [readmePreview, setReadmePreview] = useState<string | null>(null);
  const [readmePath, setReadmePath] = useState<string | null>(null);
  const [isLoadingCode, setLoadingCode] = useState(false);

  const { mutate: deleteRepository, isLoading: isDeleting } = useDelete<RepositoryView>();

  const orgQuery = useList<OrganizationView>({
    resource: "organizations",
  });
  const repoQuery = useList<RepositoryView>({
    resource: "my-repositories",
    meta: {
      organization_id: organizationId,
    },
    queryOptions: {
      enabled: Boolean(organizationId),
    },
  });

  const orgs = orgQuery.data?.data ?? [];
  const repos = repoQuery.data?.data ?? [];
  const organization = useMemo(
    () => orgs.find((item) => item.id === organizationId),
    [organizationId, orgs],
  );
  const repository = useMemo(
    () => repos.find((item) => item.id === repoId),
    [repoId, repos],
  );

  const loadBranches = async () => {
    setLoadingBranches(true);
    try {
      const data = await apiRequest<RepositoryBranchView[]>(`/repos/${repoId}/branches`);
      setBranches(data);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingBranches(false);
    }
  };

  const loadCommits = async () => {
    setLoadingCommits(true);
    try {
      const query = new URLSearchParams();
      query.set("limit", "50");
      if (branchFilter !== "all") {
        query.set("branch_name", branchFilter);
      }
      const data = await apiRequest<RepositoryCommitView[]>(`/repos/${repoId}/commits?${query.toString()}`);
      setCommits(data);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingCommits(false);
    }
  };

  const loadTree = async (branchName: string, path: string) => {
    setLoadingCode(true);
    try {
      const query = new URLSearchParams();
      query.set("branch_name", branchName);
      if (path) {
        query.set("path", path);
      }
      const data = await apiRequest<RepositoryTreeEntryView[]>(`/repos/${repoId}/tree?${query.toString()}`);
      setTreeEntries(data);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingCode(false);
    }
  };

  const loadReadmePreview = async (branchName: string) => {
    try {
      const query = new URLSearchParams();
      query.set("branch_name", branchName);
      const blob = await apiRequest<RepositoryBlobView>(`/repos/${repoId}/readme?${query.toString()}`);
      setReadmePath(blob.path);
      if (blob.is_binary) {
        setReadmePreview(`<p>${t("README is binary and cannot be rendered.")}</p>`);
        return;
      }
      const html = await renderMarkdown(blob.content);
      setReadmePreview(html);
    } catch (error) {
      const message = extractErrorMessage(error);
      if (message.toLowerCase().includes("readme not found")) {
        setReadmePath(null);
        setReadmePreview(null);
        return;
      }
      setActionError(message);
    }
  };

  const openFile = async (path: string) => {
    if (!codeBranch) {
      return;
    }
    setActionError(null);
    setLoadingCode(true);
    try {
      const query = new URLSearchParams();
      query.set("branch_name", codeBranch);
      query.set("path", path);
      const blob = await apiRequest<RepositoryBlobView>(`/repos/${repoId}/blob?${query.toString()}`);
      setSelectedBlob(blob);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingCode(false);
    }
  };

  useEffect(() => {
    if (!repoId) {
      return;
    }
    void loadBranches();
  }, [repoId]);

  useEffect(() => {
    if (!repoId) {
      return;
    }
    void loadCommits();
  }, [repoId, branchFilter]);

  useEffect(() => {
    if (!repository || codeBranch) {
      return;
    }
    setCodeBranch(repository.default_branch);
  }, [repository, codeBranch]);

  useEffect(() => {
    if (!repoId || !codeBranch) {
      return;
    }
    void loadTree(codeBranch, treePath);
  }, [repoId, codeBranch, treePath]);

  useEffect(() => {
    if (!repoId || !codeBranch || activeTab !== "code" || selectedBlob) {
      return;
    }
    void loadReadmePreview(codeBranch);
  }, [repoId, codeBranch, activeTab, selectedBlob]);

  const copyCloneUrl = async () => {
    if (!repository) {
      return;
    }
    try {
      await navigator.clipboard.writeText(repository.clone_http_url);
    } catch {
      setActionError(t("Failed to copy clone URL"));
    }
  };

  const submitCreateBranch = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!newBranchName.trim()) {
      return;
    }
    setActionError(null);
    setCreatingBranch(true);
    try {
      await apiRequest(`/repos/${repoId}/branches`, {
        method: "POST",
        body: JSON.stringify({ name: newBranchName.trim() }),
      });
      setNewBranchName("");
      await Promise.all([loadBranches(), loadCommits()]);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setCreatingBranch(false);
    }
  };

  const toggleBranchProtection = async (branch: RepositoryBranchView, protect: boolean) => {
    setActionError(null);
    setUpdatingBranch(true);
    try {
      const op = protect ? "protect" : "unprotect";
      await apiRequest(`/repos/${repoId}/branches/${encodeURIComponent(branch.name)}/${op}`, {
        method: "POST",
      });
      await loadBranches();
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setUpdatingBranch(false);
    }
  };

  const submitDelete = () => {
    if (!repository) {
      return;
    }
    const confirmText = t("Delete repository \"{name}\"?").replace("{name}", repository.name);
    if (!window.confirm(confirmText)) {
      return;
    }
    setActionError(null);
    deleteRepository(
      {
        resource: "my-repositories",
        id: repository.id,
      },
      {
        onSuccess: () => {
          navigate("/app/repositories", { replace: true });
        },
        onError: (error) => {
          setActionError(extractErrorMessage(error));
        },
      },
    );
  };

  const changeCodeBranch = (nextBranch: string) => {
    setCodeBranch(nextBranch);
    setTreePath("");
    setSelectedBlob(null);
    setReadmePreview(null);
    setReadmePath(null);
  };

  const enterDirectory = (path: string) => {
    setTreePath(path);
    setSelectedBlob(null);
  };

  const pathSegments = treePath ? treePath.split("/") : [];
  const isLoading = orgQuery.isLoading || repoQuery.isLoading;
  const errorMessage = actionError
    ?? (orgQuery.error instanceof Error
      ? orgQuery.error.message
      : repoQuery.error instanceof Error
        ? repoQuery.error.message
        : null);
  const editorTheme = document.documentElement.classList.contains("dark") ? "vs-dark" : "vs";

  return (
    <div className="space-y-4 page-enter">
      <div className="text-sm text-muted-foreground">
        <Link to="/app/repositories" className="underline underline-offset-4">
          {t("My Repositories")}
        </Link>
      </div>

      <Card className="card-enter">
        <CardHeader>
          <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
            <div className="min-w-0 space-y-2">
              <CardTitle className="truncate text-xl">
                {organization?.name ?? t("Organization")} / {repository?.name ?? t("Repository")}
              </CardTitle>
              <CardDescription className="break-all">
                {repository?.description || t("No description provided.")}
              </CardDescription>
              <div className="flex flex-wrap gap-2">
                <Badge variant="outline">{repository?.visibility ?? t("N/A")}</Badge>
                <Badge variant="secondary">
                  {t("default branch:")} {repository?.default_branch ?? "main"}
                </Badge>
                {repository ? <Badge variant="outline">{repository.key}</Badge> : null}
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button type="button" variant="outline" className="action-pop" onClick={copyCloneUrl} disabled={!repository}>
                {t("Copy Clone URL")}
              </Button>
              <Button type="button" variant="outline" className="action-pop" asChild>
                <Link to="/app/repositories">{t("Back to repositories")}</Link>
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {(["code", "commits", "branches", "settings"] as const).map((tab) => (
              <Button
                key={tab}
                type="button"
                variant={activeTab === tab ? "default" : "outline"}
                size="sm"
                onClick={() => setActiveTab(tab)}
                className="action-pop"
              >
                {t(tab[0].toUpperCase() + tab.slice(1))}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {errorMessage ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {errorMessage}
        </p>
      ) : null}

      {isLoading ? <p className="text-sm text-muted-foreground">{t("Loading...")}</p> : null}

      {!isLoading && !repository ? (
        <Card>
          <CardContent className="pt-6">
            <p className="text-sm text-muted-foreground">{t("Repository not found in selected organization.")}</p>
          </CardContent>
        </Card>
      ) : null}

      {repository && activeTab === "code" ? (
        <Card className="card-enter">
          <CardHeader>
            <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
              <div>
                <CardTitle>{t("Code")}</CardTitle>
                <CardDescription>{t("Repository source view and clone information.")}</CardDescription>
              </div>
              <div className="space-y-2">
                <Label htmlFor="code-branch-select">{t("Branch")}</Label>
                <select
                  id="code-branch-select"
                  className="h-9 w-full rounded-md border bg-background px-3 text-sm"
                  value={codeBranch}
                  onChange={(event) => changeCodeBranch(event.target.value)}
                >
                  {branches.map((branch) => (
                    <option key={branch.name} value={branch.name}>
                      {branch.name}
                    </option>
                  ))}
                  {branches.length === 0 && repository.default_branch ? (
                    <option value={repository.default_branch}>{repository.default_branch}</option>
                  ) : null}
                </select>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="rounded-md border bg-muted/40 px-3 py-2">
              <p className="text-xs text-muted-foreground">{t("Clone URL")}</p>
              <p className="break-all font-mono text-xs">{repository.clone_http_url}</p>
            </div>

            <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
              <Button type="button" size="sm" variant="outline" onClick={() => enterDirectory("")}>
                /
              </Button>
              {pathSegments.map((segment, index) => {
                const next = pathSegments.slice(0, index + 1).join("/");
                return (
                  <Button
                    key={next}
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => enterDirectory(next)}
                  >
                    {segment}
                  </Button>
                );
              })}
              {treePath ? (
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  onClick={() => enterDirectory(pathSegments.slice(0, -1).join("/"))}
                >
                  {t("Up")}
                </Button>
              ) : null}
            </div>

            <div className="grid gap-3 lg:grid-cols-[300px_1fr]">
              <div className="space-y-2 rounded-md border p-2">
                {isLoadingCode ? <p className="px-2 py-1 text-xs text-muted-foreground">{t("Loading files...")}</p> : null}
                {treeEntries.map((entry) => (
                  <Button
                    key={entry.path}
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="w-full justify-start"
                    onClick={() => (entry.kind === "tree" ? enterDirectory(entry.path) : void openFile(entry.path))}
                  >
                    {entry.kind === "tree" ? <FolderTree className="size-4" /> : <FileCode2 className="size-4" />}
                    {entry.name}
                  </Button>
                ))}
                {!isLoadingCode && treeEntries.length === 0 ? (
                  <p className="px-2 py-1 text-xs text-muted-foreground">{t("No files found.")}</p>
                ) : null}
              </div>

              <div className="rounded-md border">
                {selectedBlob ? (
                  selectedBlob.is_binary ? (
                    <div className="p-4 text-sm text-muted-foreground">
                      {t("This file is binary and cannot be shown in Monaco.")}
                    </div>
                  ) : (
                    <Editor
                      height="68vh"
                      language={detectLanguage(selectedBlob.path)}
                      value={selectedBlob.content}
                      theme={editorTheme}
                      options={{
                        readOnly: true,
                        minimap: { enabled: false },
                        fontSize: 13,
                        wordWrap: "on",
                        scrollBeyondLastLine: false,
                      }}
                    />
                  )
                ) : readmePreview ? (
                  <div className="p-4">
                    <div className="mb-3 flex items-center justify-between">
                      <p className="text-xs text-muted-foreground">{t("README Preview")}</p>
                      {readmePath ? (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          onClick={() => void openFile(readmePath)}
                        >
                          {t("View README source")}
                        </Button>
                      ) : null}
                    </div>
                    <article className="markdown-body" dangerouslySetInnerHTML={{ __html: readmePreview }} />
                  </div>
                ) : (
                  <div className="p-4 text-sm text-muted-foreground">
                    {t("README not found. Select a source file from the left to open in Monaco.")}
                  </div>
                )}
              </div>
            </div>
          </CardContent>
        </Card>
      ) : null}

      {repository && activeTab === "commits" ? (
        <Card className="card-enter">
          <CardHeader>
            <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
              <div>
                <CardTitle>{t("Commits")}</CardTitle>
                <CardDescription>{t("Recent commit activity in this repository.")}</CardDescription>
              </div>
              <div className="space-y-2">
                <Label htmlFor="branch-filter">{t("Branch")}</Label>
                <select
                  id="branch-filter"
                  className="h-9 w-full rounded-md border bg-background px-3 text-sm"
                  value={branchFilter}
                  onChange={(event) => setBranchFilter(event.target.value)}
                >
                  <option value="all">{t("All branches")}</option>
                  {branches.map((branch) => (
                    <option key={branch.name} value={branch.name}>
                      {branch.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            {isLoadingCommits ? <p className="text-sm text-muted-foreground">{t("Loading commits...")}</p> : null}
            {commits.map((commit) => (
              <div key={commit.commit_sha} className="rounded-md border p-3">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant="outline">{shortSha(commit.commit_sha)}</Badge>
                  <Badge variant="secondary">{commit.branch_name}</Badge>
                </div>
                <p className="mt-2 text-sm font-medium">{commit.message}</p>
                <p className="mt-1 text-xs text-muted-foreground">
                  {commit.author_user_id} · {formatTime(commit.created_at)}
                </p>
              </div>
            ))}
            {!isLoadingCommits && commits.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t("No commits found.")}</p>
            ) : null}
          </CardContent>
        </Card>
      ) : null}

      {repository && activeTab === "branches" ? (
        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("Branches")}</CardTitle>
            <CardDescription>{t("Manage repository branches and protections.")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <form className="grid gap-2 md:grid-cols-[1fr_auto]" onSubmit={submitCreateBranch}>
              <Input
                placeholder={t("New branch name")}
                value={newBranchName}
                onChange={(event) => setNewBranchName(event.target.value)}
              />
              <Button type="submit" disabled={isCreatingBranch || isUpdatingBranch}>
                {isCreatingBranch ? t("Creating...") : t("Create branch")}
              </Button>
            </form>

            {isLoadingBranches ? <p className="text-sm text-muted-foreground">{t("Loading branches...")}</p> : null}

            <div className="space-y-2">
              {branches.map((branch) => (
                <div key={branch.name} className="flex flex-col gap-3 rounded-md border p-3 md:flex-row md:items-center md:justify-between">
                  <div className="min-w-0">
                    <p className="font-medium">{branch.name}</p>
                    <p className="truncate text-xs text-muted-foreground">
                      {t("Last commit")}: {branch.last_commit_sha ? shortSha(branch.last_commit_sha) : t("N/A")}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={branch.is_protected ? "default" : "outline"}>
                      {branch.is_protected ? t("Protected") : t("Unprotected")}
                    </Badge>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={isUpdatingBranch}
                      onClick={() => toggleBranchProtection(branch, !branch.is_protected)}
                    >
                      {branch.is_protected ? t("Unprotect") : t("Protect")}
                    </Button>
                  </div>
                </div>
              ))}
              {!isLoadingBranches && branches.length === 0 ? (
                <p className="text-sm text-muted-foreground">{t("No branches found.")}</p>
              ) : null}
            </div>
          </CardContent>
        </Card>
      ) : null}

      {repository && activeTab === "settings" ? (
        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("Settings")}</CardTitle>
            <CardDescription>{t("Repository metadata and danger zone.")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="rounded-md border p-3 text-sm">
              <p>
                <span className="text-muted-foreground">{t("Repository key")}:</span> {repository.key}
              </p>
              <p>
                <span className="text-muted-foreground">{t("Visibility")}:</span> {repository.visibility}
              </p>
              <p>
                <span className="text-muted-foreground">{t("Default branch")}:</span> {repository.default_branch}
              </p>
            </div>

            <div className="rounded-md border border-destructive/40 bg-destructive/10 p-3">
              <p className="text-sm font-medium text-destructive">{t("Danger zone")}</p>
              <p className="mt-1 text-xs text-destructive/80">
                {t("Deleting a repository is irreversible.")}
              </p>
              <Button
                type="button"
                variant="destructive"
                size="sm"
                className="mt-3"
                disabled={isDeleting}
                onClick={submitDelete}
              >
                {t("Delete")}
              </Button>
            </div>
          </CardContent>
        </Card>
      ) : null}
    </div>
  );
}
