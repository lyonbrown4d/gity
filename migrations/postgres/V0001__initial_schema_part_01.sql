-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE audit_revisions (
    id bigint NOT NULL,
    created_at timestamp without time zone NOT NULL,
    actor text,
    tenant text,
    reason text,
    metadata text,
    CONSTRAINT audit_revisions_pkey PRIMARY KEY (id)
);

CREATE INDEX ix_audit_revisions_actor ON audit_revisions USING btree (actor);

CREATE INDEX ix_audit_revisions_created_at ON audit_revisions USING btree (created_at);

CREATE TABLE auth_sessions (
    token_hash character varying(64) NOT NULL,
    user_id bigint NOT NULL,
    kind character varying(16) NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    family character varying(64) NOT NULL,
    CONSTRAINT auth_sessions_pkey PRIMARY KEY (token_hash)
);

CREATE TABLE bootstrap_claim (
    id bigint NOT NULL,
    CONSTRAINT bootstrap_claim_pkey PRIMARY KEY (id)
);

CREATE TABLE organizations (
    id bigint NOT NULL,
    name text NOT NULL,
    path_key text NOT NULL,
    full_path text NOT NULL,
    description text,
    visibility text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT organizations_full_path_key UNIQUE (full_path),
    CONSTRAINT organizations_path_key_key UNIQUE (path_key),
    CONSTRAINT organizations_pkey PRIMARY KEY (id)
);

CREATE TABLE user_passwords (
    user_id bigint NOT NULL,
    password_hash text NOT NULL,
    CONSTRAINT user_passwords_pkey PRIMARY KEY (user_id)
);

CREATE TABLE users (
    id bigint NOT NULL,
    username text NOT NULL,
    display_name text NOT NULL,
    email text,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    is_super_admin integer DEFAULT 0 NOT NULL,
    status text DEFAULT 'active'::text NOT NULL,
    CONSTRAINT users_email_key UNIQUE (email),
    CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_username_key UNIQUE (username)
);

CREATE TABLE organization_members (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    user_id bigint NOT NULL,
    role text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT organization_members_pkey PRIMARY KEY (id),
    CONSTRAINT organization_members_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT organization_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX ix_organization_members_user ON organization_members USING btree (user_id);

CREATE UNIQUE INDEX ux_organization_members_org_user ON organization_members USING btree (organization_id, user_id);

CREATE TABLE project_branch_protection_audits (
    revision_id bigint NOT NULL,
    operation text NOT NULL,
    project_branch_protection_id bigint CONSTRAINT project_branch_protection_a_project_branch_protection__not_null NOT NULL,
    project_id bigint NOT NULL,
    branch_name text NOT NULL,
    rule_type text NOT NULL,
    push_access_level text NOT NULL,
    merge_access_level text NOT NULL,
    require_merge_request integer NOT NULL,
    require_pipeline_success integer CONSTRAINT project_branch_protection_aud_require_pipeline_success_not_null NOT NULL,
    allow_force_push integer NOT NULL,
    allow_delete integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_branch_protection_audits_revision_id_fkey FOREIGN KEY (revision_id) REFERENCES audit_revisions(id)
);

CREATE INDEX ix_project_branch_protection_audits_entity ON project_branch_protection_audits USING btree (project_branch_protection_id);

CREATE INDEX ix_project_branch_protection_audits_project ON project_branch_protection_audits USING btree (project_id);

CREATE INDEX ix_project_branch_protection_audits_revision ON project_branch_protection_audits USING btree (revision_id);

CREATE TABLE project_ci_variable_audits (
    revision_id bigint NOT NULL,
    operation text NOT NULL,
    project_ci_variable_id bigint NOT NULL,
    project_id bigint NOT NULL,
    key text NOT NULL,
    masked integer NOT NULL,
    protected integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_ci_variable_audits_revision_id_fkey FOREIGN KEY (revision_id) REFERENCES audit_revisions(id)
);

CREATE INDEX ix_project_ci_variable_audits_entity ON project_ci_variable_audits USING btree (project_ci_variable_id);

CREATE INDEX ix_project_ci_variable_audits_key ON project_ci_variable_audits USING btree (key);

CREATE INDEX ix_project_ci_variable_audits_project ON project_ci_variable_audits USING btree (project_id);

CREATE INDEX ix_project_ci_variable_audits_revision ON project_ci_variable_audits USING btree (revision_id);

CREATE TABLE project_deploy_key_audits (
    revision_id bigint NOT NULL,
    operation text NOT NULL,
    project_deploy_key_id bigint NOT NULL,
    project_id bigint NOT NULL,
    title text NOT NULL,
    fingerprint text NOT NULL,
    can_push integer NOT NULL,
    created_by_user_id bigint NOT NULL,
    last_used_at timestamp without time zone,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_deploy_key_audits_revision_id_fkey FOREIGN KEY (revision_id) REFERENCES audit_revisions(id)
);

CREATE INDEX ix_project_deploy_key_audits_entity ON project_deploy_key_audits USING btree (project_deploy_key_id);

CREATE INDEX ix_project_deploy_key_audits_fingerprint ON project_deploy_key_audits USING btree (fingerprint);

CREATE INDEX ix_project_deploy_key_audits_project ON project_deploy_key_audits USING btree (project_id);

CREATE INDEX ix_project_deploy_key_audits_revision ON project_deploy_key_audits USING btree (revision_id);

CREATE TABLE project_member_audits (
    revision_id bigint NOT NULL,
    operation text NOT NULL,
    project_member_id bigint NOT NULL,
    project_id bigint NOT NULL,
    user_id bigint NOT NULL,
    role text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_member_audits_revision_id_fkey FOREIGN KEY (revision_id) REFERENCES audit_revisions(id)
);

CREATE INDEX ix_project_member_audits_entity ON project_member_audits USING btree (project_member_id);

CREATE INDEX ix_project_member_audits_project ON project_member_audits USING btree (project_id);

CREATE INDEX ix_project_member_audits_revision ON project_member_audits USING btree (revision_id);

CREATE INDEX ix_project_member_audits_user ON project_member_audits USING btree (user_id);

CREATE TABLE project_merge_request_approval_rule_audits (
    revision_id bigint NOT NULL,
    operation text NOT NULL,
    project_merge_request_approval_rule_id bigint CONSTRAINT project_merge_request_appro_project_merge_request_appr_not_null NOT NULL,
    project_id bigint NOT NULL,
    name text NOT NULL,
    target_branch text CONSTRAINT project_merge_request_approval_rule_audi_target_branch_not_null NOT NULL,
    approvals_required integer CONSTRAINT project_merge_request_approval_rul_approvals_required_not_null1 NOT NULL,
    eligible_user_ids text CONSTRAINT project_merge_request_approval_rule__eligible_user_ids_not_null NOT NULL,
    code_owner integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_merge_request_approval_rule_audits_revision_id_fkey FOREIGN KEY (revision_id) REFERENCES audit_revisions(id)
);

CREATE INDEX ix_project_mr_approval_rule_audits_entity ON project_merge_request_approval_rule_audits USING btree (project_merge_request_approval_rule_id);

CREATE INDEX ix_project_mr_approval_rule_audits_project ON project_merge_request_approval_rule_audits USING btree (project_id);

CREATE INDEX ix_project_mr_approval_rule_audits_revision ON project_merge_request_approval_rule_audits USING btree (revision_id);

CREATE TABLE projects (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    name text NOT NULL,
    path_key text NOT NULL,
    full_path text NOT NULL,
    visibility text NOT NULL,
    description text,
    default_branch text NOT NULL,
    status text NOT NULL,
    deleted_at timestamp without time zone,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT projects_full_path_key UNIQUE (full_path),
    CONSTRAINT projects_pkey PRIMARY KEY (id),
    CONSTRAINT projects_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);

CREATE INDEX ix_projects_organization_status ON projects USING btree (organization_id, status);

CREATE INDEX ix_projects_visibility ON projects USING btree (visibility);

CREATE TABLE user_access_tokens (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    name text NOT NULL,
    token text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT user_access_tokens_pkey PRIMARY KEY (id),
    CONSTRAINT user_access_tokens_token_key UNIQUE (token),
    CONSTRAINT user_access_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
