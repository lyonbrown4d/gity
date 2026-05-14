export type RepoTab = "code" | "issues" | "merge-requests" | "wiki" | "packages" | "releases" | "lfs" | "pipelines" | "jobs" | "runners" | "commits" | "branches" | "audit" | "settings";

export interface RepositoryTreeNode {
  name: string;
  path: string;
  kind: string;
  children: RepositoryTreeNode[];
  expanded: boolean;
  loaded: boolean;
  loading: boolean;
}
