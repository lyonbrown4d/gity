export interface OrganizationView {
  id: string;
  key: string;
  name: string;
  role: string;
}

export interface RepositoryView {
  id: string;
  organization_id: string;
  key: string;
  name: string;
  description?: string;
  visibility: string;
  default_branch: string;
  clone_http_url: string;
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
