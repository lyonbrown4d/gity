import { useEffect, useState } from "react";
import { useCustomMutation, useDataProvider, type BaseRecord } from "@refinedev/core";
import type {
  RepositoryBlobView,
  RepositoryBranchView,
  RepositoryLanguagesView,
  RepositoryTreeEntryView,
} from "@/pages/types";
import { findTreeNode, patchTreeNode, toTreeNodes } from "./repository-tree";
import type { RepositoryTreeNode } from "./repository-types";
import { extractErrorMessage, normalizeRepoFilePath, renderMarkdown } from "./repository-utils";

interface UseRepositorySourceControllerArgs {
  repoId: string;
  activeTab: "code" | "issues" | "commits" | "branches" | "settings";
  defaultBranch: string | null;
  branches: RepositoryBranchView[];
  t: (text: string) => string;
  onError: (message: string | null) => void;
  refreshBranches: () => Promise<void>;
  refreshCommits: () => Promise<void>;
}

export const useRepositorySourceController = ({
  repoId,
  activeTab,
  defaultBranch,
  branches,
  t,
  onError,
  refreshBranches,
  refreshCommits,
}: UseRepositorySourceControllerArgs) => {
  const dataProvider = useDataProvider();
  const { mutateAsync: createFileCommit, isLoading: isCreatingFile } = useCustomMutation();
  const [codeBranch, setCodeBranch] = useState("");
  const [treeNodes, setTreeNodes] = useState<RepositoryTreeNode[]>([]);
  const [selectedBlob, setSelectedBlob] = useState<RepositoryBlobView | null>(null);
  const [readmePreview, setReadmePreview] = useState<string | null>(null);
  const [readmePath, setReadmePath] = useState<string | null>(null);
  const [isLoadingTree, setLoadingTree] = useState(false);
  const [isCreateFileModalOpen, setCreateFileModalOpen] = useState(false);
  const [newFileBranch, setNewFileBranch] = useState("");
  const [newFilePath, setNewFilePath] = useState("");
  const [newFileMessage, setNewFileMessage] = useState("");
  const [newFileContent, setNewFileContent] = useState("");
  const [languages, setLanguages] = useState<RepositoryLanguagesView | null>(null);
  const [isLoadingLanguages, setLoadingLanguages] = useState(false);

  const request = async <T extends BaseRecord = BaseRecord>(
    url: string,
    method: "get" | "delete" | "head" | "options" | "post" | "put" | "patch" = "get",
    options?: { query?: Record<string, string>; payload?: unknown },
  ): Promise<T> => {
    const custom = dataProvider().custom;
    if (!custom) {
      throw new Error("dataProvider.custom is not configured");
    }

    const response = await custom<T>({
      url,
      method,
      query: options?.query,
      payload: options?.payload,
    });
    return response.data;
  };

  const loadLanguages = async (branchName: string) => {
    if (!branchName) {
      return;
    }
    setLoadingLanguages(true);
    try {
      setLanguages(await request<RepositoryLanguagesView>(`/repos/${repoId}/languages`, "get", {
        query: { branch_name: branchName },
      }));
    } catch (error) {
      onError(extractErrorMessage(error));
    } finally {
      setLoadingLanguages(false);
    }
  };

  const fetchTreeEntries = async (branchName: string, path?: string): Promise<RepositoryTreeEntryView[]> => {
    const query: Record<string, string> = { branch_name: branchName };
    if (path) {
      query.path = path;
    }
    return request<RepositoryTreeEntryView[]>(`/repos/${repoId}/tree`, "get", { query });
  };

  const loadTreeRoot = async (branchName: string) => {
    setLoadingTree(true);
    try {
      setTreeNodes(toTreeNodes(await fetchTreeEntries(branchName)));
    } catch (error) {
      onError(extractErrorMessage(error));
    } finally {
      setLoadingTree(false);
    }
  };

  const toggleTreeDirectory = async (path: string) => {
    const node = findTreeNode(treeNodes, path);
    if (!node || node.kind !== "tree" || !codeBranch) {
      return;
    }
    if (node.expanded) {
      setTreeNodes((current) => patchTreeNode(current, path, (target) => ({ ...target, expanded: false })));
      return;
    }
    if (node.loaded) {
      setTreeNodes((current) => patchTreeNode(current, path, (target) => ({ ...target, expanded: true })));
      return;
    }
    setTreeNodes((current) => patchTreeNode(current, path, (target) => ({ ...target, expanded: true, loading: true })));
    try {
      const entries = await fetchTreeEntries(codeBranch, path);
      setTreeNodes((current) => patchTreeNode(current, path, (target) => ({
        ...target,
        expanded: true,
        loaded: true,
        loading: false,
        children: toTreeNodes(entries),
      })));
    } catch (error) {
      setTreeNodes((current) => patchTreeNode(current, path, (target) => ({ ...target, loading: false })));
      onError(extractErrorMessage(error));
    }
  };

  const loadReadmePreview = async (branchName: string) => {
    try {
      const blob = await request<RepositoryBlobView>(`/repos/${repoId}/readme`, "get", {
        query: { branch_name: branchName },
      });
      setReadmePath(blob.path);
      if (blob.is_binary) {
        setReadmePreview(`<p>${t("README is binary and cannot be rendered.")}</p>`);
        return;
      }
      setReadmePreview(await renderMarkdown(blob.content));
    } catch (error) {
      const message = extractErrorMessage(error);
      if (message.toLowerCase().includes("readme not found")) {
        setReadmePath(null);
        setReadmePreview(null);
      } else {
        onError(message);
      }
    }
  };

  const openFile = async (path: string) => {
    if (!codeBranch) {
      return;
    }
    onError(null);
    try {
      setSelectedBlob(await request<RepositoryBlobView>(`/repos/${repoId}/blob`, "get", {
        query: { branch_name: codeBranch, path },
      }));
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const changeCodeBranch = (nextBranch: string) => {
    setCodeBranch(nextBranch);
    setTreeNodes([]);
    setSelectedBlob(null);
    setReadmePreview(null);
    setReadmePath(null);
    setLanguages(null);
  };

  const openCreateFileModal = () => {
    const fallbackBranch = codeBranch || defaultBranch || branches[0]?.name || "main";
    setNewFileBranch(fallbackBranch);
    setNewFilePath("");
    setNewFileMessage(t("Add new file"));
    setNewFileContent("");
    setCreateFileModalOpen(true);
  };

  const submitCreateFileCommit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const branchName = newFileBranch.trim();
    const filePath = normalizeRepoFilePath(newFilePath);
    const message = newFileMessage.trim();
    if (!branchName) return onError(t("Branch is required"));
    if (!filePath) return onError(t("File path is required"));
    if (!message) return onError(t("Commit message is required"));

    onError(null);
    try {
      await createFileCommit({
        url: `/repos/${repoId}/file-commits`,
        method: "post",
        values: { branch_name: branchName, path: filePath, content: newFileContent, message },
      });
      setCreateFileModalOpen(false);
      setCodeBranch(branchName);
      await Promise.all([refreshBranches(), refreshCommits(), loadTreeRoot(branchName), loadLanguages(branchName)]);
      setSelectedBlob(await request<RepositoryBlobView>(`/repos/${repoId}/blob`, "get", {
        query: { branch_name: branchName, path: filePath },
      }));
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    if (defaultBranch && !codeBranch) {
      setCodeBranch(defaultBranch);
    }
  }, [defaultBranch, codeBranch]);

  useEffect(() => {
    if (repoId && codeBranch) {
      void loadTreeRoot(codeBranch);
    }
  }, [repoId, codeBranch]);

  useEffect(() => {
    if (repoId && codeBranch) {
      void loadLanguages(codeBranch);
    }
  }, [repoId, codeBranch]);

  useEffect(() => {
    if (repoId && codeBranch && activeTab === "code" && !selectedBlob) {
      void loadReadmePreview(codeBranch);
    }
  }, [repoId, codeBranch, activeTab, selectedBlob]);

  return {
    codeBranch,
    treeNodes,
    selectedBlob,
    readmePreview,
    readmePath,
    isLoadingTree,
    isCreateFileModalOpen,
    newFileBranch,
    newFilePath,
    newFileMessage,
    newFileContent,
    isCreatingFile,
    languages,
    isLoadingLanguages,
    setCreateFileModalOpen,
    setNewFileBranch,
    setNewFilePath,
    setNewFileMessage,
    setNewFileContent,
    loadLanguages,
    openFile,
    toggleTreeDirectory,
    openCreateFileModal,
    submitCreateFileCommit,
    changeCodeBranch,
  };
};
