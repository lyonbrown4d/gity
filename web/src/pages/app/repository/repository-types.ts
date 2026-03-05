export type RepoTab = "code" | "issues" | "commits" | "branches" | "settings";

export interface RepositoryTreeNode {
  name: string;
  path: string;
  kind: string;
  children: RepositoryTreeNode[];
  expanded: boolean;
  loaded: boolean;
  loading: boolean;
}
