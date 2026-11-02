import type { RepositoryTreeEntryView } from "@/pages/types";
import type { RepositoryTreeNode } from "./repository-types";

export const toTreeNodes = (entries: RepositoryTreeEntryView[]): RepositoryTreeNode[] => {
  return entries.map((entry) => ({
    name: entry.name,
    path: entry.path,
    kind: entry.kind,
    children: [],
    expanded: false,
    loaded: entry.kind !== "tree",
    loading: false,
  }));
};

export const patchTreeNode = (
  nodes: RepositoryTreeNode[],
  targetPath: string,
  updater: (node: RepositoryTreeNode) => RepositoryTreeNode,
): RepositoryTreeNode[] => {
  return nodes.map((node) => {
    if (node.path === targetPath) {
      return updater(node);
    }
    if (node.children.length === 0) {
      return node;
    }
    return {
      ...node,
      children: patchTreeNode(node.children, targetPath, updater),
    };
  });
};

export const findTreeNode = (
  nodes: RepositoryTreeNode[],
  targetPath: string,
): RepositoryTreeNode | null => {
  for (const node of nodes) {
    if (node.path === targetPath) {
      return node;
    }
    if (node.children.length > 0) {
      const found = findTreeNode(node.children, targetPath);
      if (found) {
        return found;
      }
    }
  }
  return null;
};
