export interface OrganizationView {
  id: string;
  key: string;
  name: string;
  role: string;
}

export interface RepositoryView {
  id: string;
  uuid: string;
  organization_id: string;
  key: string;
  name: string;
  description?: string;
  visibility: string;
  default_branch: string;
  clone_http_url: string;
}

export interface RepositoryBranchView {
  repository_id: string;
  name: string;
  is_protected: boolean;
  last_commit_sha?: string | null;
}

export interface RepositoryCommitView {
  repository_id: string;
  branch_name: string;
  commit_sha: string;
  message: string;
  author_user_id: string;
  created_at: string;
}

export interface RepositoryTreeEntryView {
  name: string;
  path: string;
  kind: string;
  oid: string;
  size?: number;
}

export interface RepositoryBlobView {
  path: string;
  content: string;
  size: number;
  is_binary: boolean;
  encoding: string;
}

export interface RepositoryLanguageItemView {
  language: string;
  bytes: number;
  percentage: number;
}

export interface RepositoryLanguagesView {
  repository_id: string;
  branch_name: string;
  status: string;
  revision?: string | null;
  analyzed_at?: string | null;
  total_bytes: number;
  languages: RepositoryLanguageItemView[];
}

export interface RepositoryIssueView {
  id: string;
  repository_id: string;
  number: number;
  title: string;
  description?: string | null;
  status: "open" | "closed";
  author_user_id: string;
  assignee_user_id?: string | null;
  created_at: string;
  updated_at: string;
  closed_at?: string | null;
}

export interface RepositoryIssueCommentView {
  id: string;
  issue_id: string;
  author_user_id: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface IssueAttachmentUploadView {
  url: string;
  object_key: string;
  file_name: string;
  content_type: string;
  size: number;
  markdown: string;
}

export interface UserView {
  id: string;
  username: string;
  email: string;
  status: string;
  is_super_admin: boolean;
}

export interface OrganizationMemberView {
  organization_id: string;
  user_id: string;
  username: string;
  email: string;
  role: string;
}
