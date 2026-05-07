import { useEffect, useState } from "react";
import { useCustomMutation, useDataProvider, type BaseRecord } from "@refinedev/core";
import type {
  RepositoryBlobView,
  RepositoryBranchView,
  RepositoryLanguagesView,
  RepositoryTreeEntryView,
  RepositorySearchResultView,
} from "@/pages/types";
import { findTreeNode, patchTreeNode, toTreeNodes } from "./repository-tree";
import type { RepositoryTreeNode } from "./repository-types";
import type { RepoTab } from "./repository-types";
import { extractErrorMessage, normalizeRepoFilePath, renderMarkdown } from "./repository-utils";

interface UseRepositorySourceControllerArgs {
  repoId: string;
  activeTab: RepoTab;
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
  const [isLoadingSearch, setLoadingSearch] = useState(false);
  const [isCreateFileModalOpen, setCreateFileModalOpen] = useState(false);
  const [newFileBranch, setNewFileBranch] = useState("");
  const [newFilePath, setNewFilePath] = useState("");
  const [newFileMessage, setNewFileMessage] = useState("");
  const [newFileContent, setNewFileContent] = useState("");
  const [languages, setLanguages] = useState<RepositoryLanguagesView | null>(null);
  const [isLoadingLanguages, setLoadingLanguages] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchPath, setSearchPath] = useState("");
  const [searchMatchCase, setSearchMatchCase] = useState(false);
  const [searchRegex, setSearchRegex] = useState(false);
  const [searchResults, setSearchResults] = useState<RepositorySearchResultView[]>([]);
  const [searchLine, setSearchLine] = useState(1);
  const [searchColumn, setSearchColumn] = useState(1);
  const [searchMatchLength, setSearchMatchLength] = useState(0);
  const [searchJumpRequest, setSearchJumpRequest] = useState(0);
  const [searchTargetPath, setSearchTargetPath] = useState("");

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
      setLanguages(await request<RepositoryLanguagesView>(`/projects/${repoId}/languages`, "get", {
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
    return request<RepositoryTreeEntryView[]>(`/projects/${repoId}/repository/tree`, "get", { query });
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
      const blob = await request<RepositoryBlobView>(`/projects/${repoId}/repository/readme`, "get", {
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

  const openFile = async (path: string, keepSearchPosition = false) => {
    if (!codeBranch) {
      return;
    }
    onError(null);
    if (!keepSearchPosition) {
      setSearchTargetPath("");
      setSearchLine(1);
      setSearchColumn(1);
      setSearchMatchLength(0);
      setSearchJumpRequest(0);
    }
    try {
      setSelectedBlob(await request<RepositoryBlobView>(`/projects/${repoId}/repository/blob`, "get", {
        query: { branch_name: codeBranch, path },
      }));
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const openFileAtLine = async (path: string, lineNumber = 1, column = 1, matchLength = 0) => {
    setSearchTargetPath(path);
    setSearchLine(Math.max(lineNumber, 1));
    setSearchColumn(Math.max(column, 1));
    setSearchMatchLength(Math.max(matchLength, 0));
    setSearchJumpRequest((current) => current + 1);
    await openFile(path, true);
  };

  const searchCode = async (query: string) => {
    const trimmedQuery = query.trim();
    if (!trimmedQuery) {
      setSearchResults([]);
      return;
    }
    if (!codeBranch) {
      return;
    }
    setLoadingSearch(true);
    try {
      const filters: Record<string, string> = {
        branch_name: codeBranch,
        query: trimmedQuery,
        match_case: String(searchMatchCase),
        regex: String(searchRegex),
      };
      const normalizedPath = normalizeRepoFilePath(searchPath);
      if (normalizedPath) {
        filters.path = normalizedPath;
      }
      setSearchResults(await request<RepositorySearchResultView[]>(`/projects/${repoId}/repository/search`, "get", {
        query: filters,
      }));
    } catch (error) {
      onError(extractErrorMessage(error));
    } finally {
      setLoadingSearch(false);
    }
  };

  const changeCodeBranch = (nextBranch: string) => {
    setCodeBranch(nextBranch);
    setTreeNodes([]);
    setSelectedBlob(null);
    setReadmePreview(null);
    setReadmePath(null);
    setLanguages(null);
    setSearchResults([]);
    setSearchQuery("");
    setSearchPath("");
    setSearchMatchCase(false);
    setSearchRegex(false);
    setSearchLine(1);
    setSearchColumn(1);
    setSearchMatchLength(0);
    setSearchTargetPath("");
    setSearchJumpRequest(0);
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
        url: `/projects/${repoId}/file-commits`,
        method: "post",
        values: { branch_name: branchName, path: filePath, content: newFileContent, message },
      });
      setCreateFileModalOpen(false);
      setCodeBranch(branchName);
      await Promise.all([refreshBranches(), refreshCommits(), loadTreeRoot(branchName), loadLanguages(branchName)]);
      setSearchLine(1);
      setSelectedBlob(await request<RepositoryBlobView>(`/projects/${repoId}/repository/blob`, "get", {
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
    searchJumpRequest,
    searchColumn,
    searchTargetPath,
    setCreateFileModalOpen,
    setNewFileBranch,
    setNewFilePath,
    setNewFileMessage,
    setNewFileContent,
    loadLanguages,
    openFile,
    openFileAtLine,
    toggleTreeDirectory,
    openCreateFileModal,
    submitCreateFileCommit,
    changeCodeBranch,
    searchQuery,
    setSearchQuery,
    searchPath,
    setSearchPath,
    searchMatchCase,
    setSearchMatchCase,
    searchRegex,
    setSearchRegex,
    searchLine,
    isLoadingSearch,
    searchResults,
    searchMatchLength,
    searchCode,
  };
};
