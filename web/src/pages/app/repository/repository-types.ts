export type RepoTab = "code" | "issues" | "merge-requests" | "wiki" | "packages" | "lfs" | "pipelines" | "jobs" | "runners" | "commits" | "branches" | "settings";

export interface RepositoryTreeNode {
  name: string;
  path: string;
  kind: string;
  children: RepositoryTreeNode[];
  expanded: boolean;
  loaded: boolean;
  loading: boolean;
}
