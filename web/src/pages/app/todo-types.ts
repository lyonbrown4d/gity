export type TodoState = "pending" | "done";

export type TodoKind =
  | "project_created"
  | "issue_created"
  | "issue_assigned"
  | "issue_commented"
  | "branch_protection_changed"
  | "branch_deleted"
  | "merge_request_merged"
  | "merge_request_commented"
  | "merge_request_approved"
  | "merge_request_unapproved"
  | "merge_request_review_requested"
  | "merge_request_assigned";

export interface TodoView {
  id: string;
  user_id: string;
  organization_id: string;
  project_id: string;
  kind: TodoKind;
  state: TodoState;
  target_type: string;
  target_id: string;
  title: string;
  summary: string;
  action_url: string;
  created_at: string;
  updated_at: string;
  done_at?: string;
}

export interface TodoDoneView {
  id: string;
  state: "done";
}
