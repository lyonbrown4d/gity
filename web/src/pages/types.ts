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
  full_path: string;
  name: string;
  description?: string;
  visibility: string;
  default_branch: string;
  status: "active" | "pending_delete";
  deleted_at?: string | null;
  clone_http_url: string;
}

export interface RepositoryBranchView {
  repository_id: string;
  name: string;
  is_protected: boolean;
  last_commit_sha?: string | null;
  protection?: RepositoryBranchProtectionView | null;
}

export interface RepositoryBranchProtectionView {
  id: string;
  repository_id: string;
  branch_name: string;
  rule_type: "exact" | "pattern";
  push_access_level: "no_one" | "developer" | "maintainer" | "owner";
  merge_access_level: "no_one" | "developer" | "maintainer" | "owner";
  require_merge_request: boolean;
  require_pipeline_success: boolean;
  allow_force_push: boolean;
  allow_delete: boolean;
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

export interface RepositorySearchResultView {
  path: string;
  line_number: number;
  column: number;
  match_length: number;
  line_content: string;
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

export type RepositoryMergeRequestState = "opened" | "closed" | "merged";

export interface RepositoryMergeRequestView {
  id: string;
  project_id: string;
  iid: number;
  author_user_id: string;
  title: string;
  description?: string | null;
  state: RepositoryMergeRequestState;
  source_branch: string;
  target_branch: string;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryMergeRequestDiffView {
  merge_request: RepositoryMergeRequestView;
  diff: string;
}

export type RepositoryJobStatus = "pending" | "running" | "succeeded" | "failed" | "cancelled";
export type RepositoryPipelineStatus = "pending" | "running" | "succeeded" | "failed" | "cancelled";
export type RepositoryPipelineJobStatus = RepositoryJobStatus | "blocked";

export interface RepositoryPipelineView {
  id: string;
  project_id: string;
  iid: number;
  name: string;
  source: string;
  ref_name: string;
  commit_sha: string;
  status: RepositoryPipelineStatus;
  config_source: string;
  config_content?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
  started_at?: string | null;
  finished_at?: string | null;
}

export interface RepositoryJobView {
  id: string;
  project_id: string;
  kind: string;
  status: RepositoryJobStatus;
  payload?: string | null;
  result?: string | null;
  attempts: number;
  max_attempts: number;
  run_after?: string | null;
  locked_by?: string | null;
  locked_until?: string | null;
  last_error?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
  started_at?: string | null;
  finished_at?: string | null;
}

export interface RepositoryJobTraceView {
  job: RepositoryJobView;
  trace: string;
  exit_code: number;
  output_truncated: boolean;
  duration_millis: number;
}

export interface RepositoryJobArtifactView {
  id: string;
  project_id: string;
  project_job_id: string;
  name: string;
  file_name: string;
  file_path?: string | null;
  content_type?: string | null;
  byte_size: number;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryJobArtifactContentView {
  artifact: RepositoryJobArtifactView;
  content_base64: string;
}

export interface RepositoryPipelineJobLinkView {
  id: string;
  project_id: string;
  pipeline_id: string;
  project_job_id: string;
  name: string;
  stage: string;
  needs?: string | null;
  image?: string | null;
  script?: string | null;
  artifacts?: string | null;
  sort_order: number;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryPipelineJobView {
  pipeline_job: RepositoryPipelineJobLinkView;
  project_job: RepositoryJobView;
  status: RepositoryPipelineJobStatus;
  needs: string[];
  script: string[];
  artifacts: string[];
}

export interface RepositoryPipelineDetailView {
  pipeline: RepositoryPipelineView;
  jobs: RepositoryPipelineJobView[];
}

export interface RepositoryWikiPageView {
  id: string;
  project_id: string;
  slug: string;
  title: string;
  content: string;
  format: string;
  author_user_id: string;
  last_edited_by_user_id: string;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryRunnerView {
  id: string;
  project_id: string;
  name: string;
  description?: string | null;
  tags?: string | null;
  status: "online" | "offline";
  active: boolean;
  last_contact_at?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryPackageView {
  id: string;
  project_id: string;
  type: string;
  name: string;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryPackageVersionView {
  id: string;
  project_package_id: string;
  version: string;
  status: string;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryPackageFileView {
  id: string;
  project_package_version_id: string;
  file_name: string;
  file_path?: string | null;
  content_type?: string | null;
  byte_size: number;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryPackageVersionDetailView {
  version: RepositoryPackageVersionView;
  files: RepositoryPackageFileView[];
}

export interface RepositoryPackageDetailView {
  package: RepositoryPackageView;
  versions: RepositoryPackageVersionDetailView[];
}

export interface RepositoryPackageFileContentView {
  file: RepositoryPackageFileView;
  content_base64: string;
}

export interface RepositoryLFSObjectView {
  id: string;
  project_id: string;
  oid: string;
  byte_size: number;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface RepositoryLFSLockOwnerView {
  name: string;
}

export interface RepositoryLFSLockView {
  id: string;
  path: string;
  locked_at?: string | null;
  owner: RepositoryLFSLockOwnerView;
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
