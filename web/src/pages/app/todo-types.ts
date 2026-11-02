export type TodoState = "pending" | "done";

export type TodoKind =
  | "project_created"
  | "issue_created"
  | "issue_assigned"
  | "issue_commented"
  | "branch_protection_changed"
  | "branch_deleted"
  | "merge_request_merged";

export interface TodoView {
  id: number;
  user_id: number;
  organization_id: number;
  project_id: number;
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
  id: number;
  state: "done";
}
