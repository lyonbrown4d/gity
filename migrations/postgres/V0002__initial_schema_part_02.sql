-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE project_access_tokens (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    kind text NOT NULL,
    name text NOT NULL,
    username text NOT NULL,
    token text NOT NULL,
    scopes text NOT NULL,
    created_by_user_id bigint NOT NULL,
    expires_at timestamp without time zone,
    revoked_at timestamp without time zone,
    last_used_at timestamp without time zone,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_access_tokens_pkey PRIMARY KEY (id),
    CONSTRAINT project_access_tokens_created_by_user_id_fkey FOREIGN KEY (created_by_user_id) REFERENCES users(id),
    CONSTRAINT project_access_tokens_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_access_tokens_created_by ON project_access_tokens USING btree (created_by_user_id);

CREATE INDEX ix_project_access_tokens_project_kind ON project_access_tokens USING btree (project_id, kind);

CREATE INDEX ix_project_access_tokens_username ON project_access_tokens USING btree (username);

CREATE UNIQUE INDEX ux_project_access_tokens_token ON project_access_tokens USING btree (token);

CREATE TABLE project_audit_events (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    organization_id bigint NOT NULL,
    event_name text NOT NULL,
    action text NOT NULL,
    actor_user_id bigint NOT NULL,
    target_type text NOT NULL,
    target_id text NOT NULL,
    summary text NOT NULL,
    payload text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    CONSTRAINT project_audit_events_pkey PRIMARY KEY (id),
    CONSTRAINT project_audit_events_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT project_audit_events_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_audit_events_actor ON project_audit_events USING btree (actor_user_id);

CREATE INDEX ix_project_audit_events_event_action ON project_audit_events USING btree (event_name, action);

CREATE INDEX ix_project_audit_events_project_created ON project_audit_events USING btree (project_id, created_at);

CREATE TABLE project_branch_protections (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    branch_name text NOT NULL,
    rule_type text NOT NULL,
    push_access_level text NOT NULL,
    merge_access_level text NOT NULL,
    require_merge_request integer NOT NULL,
    require_pipeline_success integer NOT NULL,
    allow_force_push integer NOT NULL,
    allow_delete integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_branch_protections_pkey PRIMARY KEY (id),
    CONSTRAINT project_branch_protections_project_id_branch_name_key UNIQUE (project_id, branch_name),
    CONSTRAINT project_branch_protections_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE project_ci_variables (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    key text NOT NULL,
    value text NOT NULL,
    masked integer NOT NULL,
    protected integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_ci_variables_pkey PRIMARY KEY (id),
    CONSTRAINT project_ci_variables_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_ci_variables_flags ON project_ci_variables USING btree (masked, protected);

CREATE UNIQUE INDEX ux_project_ci_variables_project_key ON project_ci_variables USING btree (project_id, key);

CREATE TABLE project_deploy_keys (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    title text NOT NULL,
    fingerprint text NOT NULL,
    public_key text NOT NULL,
    can_push integer NOT NULL,
    created_by_user_id bigint NOT NULL,
    last_used_at timestamp without time zone,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_deploy_keys_pkey PRIMARY KEY (id),
    CONSTRAINT project_deploy_keys_created_by_user_id_fkey FOREIGN KEY (created_by_user_id) REFERENCES users(id),
    CONSTRAINT project_deploy_keys_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_deploy_keys_created_by ON project_deploy_keys USING btree (created_by_user_id);

CREATE UNIQUE INDEX ux_project_deploy_keys_project_fingerprint ON project_deploy_keys USING btree (project_id, fingerprint);

CREATE TABLE project_iid_counters (
    project_id bigint NOT NULL,
    counter_name text NOT NULL,
    current_value bigint DEFAULT 0 NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_iid_counters_pkey PRIMARY KEY (project_id, counter_name),
    CONSTRAINT fk_project_iid_counters_project_id FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE project_issues (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    iid bigint NOT NULL,
    author_user_id bigint NOT NULL,
    title text NOT NULL,
    description text,
    state text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_issues_pkey PRIMARY KEY (id),
    CONSTRAINT project_issues_author_user_id_fkey FOREIGN KEY (author_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT project_issues_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_issues_author ON project_issues USING btree (author_user_id);

CREATE INDEX ix_project_issues_project_state ON project_issues USING btree (project_id, state);

CREATE UNIQUE INDEX ux_project_issues_project_iid ON project_issues USING btree (project_id, iid);

CREATE TABLE project_jobs (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    kind text NOT NULL,
    status text NOT NULL,
    payload text,
    result text,
    attempts integer NOT NULL,
    max_attempts integer NOT NULL,
    run_after timestamp without time zone NOT NULL,
    locked_by text,
    locked_until timestamp without time zone NOT NULL,
    last_error text,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    started_at timestamp without time zone NOT NULL,
    finished_at timestamp without time zone NOT NULL,
    CONSTRAINT project_jobs_pkey PRIMARY KEY (id),
    CONSTRAINT project_jobs_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_jobs_locked ON project_jobs USING btree (locked_by, locked_until);

CREATE INDEX ix_project_jobs_queue ON project_jobs USING btree (status, run_after, kind);

CREATE TABLE project_lfs_locks (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    owner_user_id bigint NOT NULL,
    path text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_lfs_locks_pkey PRIMARY KEY (id),
    CONSTRAINT project_lfs_locks_owner_user_id_fkey FOREIGN KEY (owner_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT project_lfs_locks_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_lfs_locks_owner ON project_lfs_locks USING btree (owner_user_id);

CREATE UNIQUE INDEX ux_project_lfs_locks_project_path ON project_lfs_locks USING btree (project_id, path);

CREATE TABLE project_lfs_objects (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    oid text NOT NULL,
    byte_size bigint NOT NULL,
    storage_key text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_lfs_objects_pkey PRIMARY KEY (id),
    CONSTRAINT project_lfs_objects_storage_key_key UNIQUE (storage_key),
    CONSTRAINT project_lfs_objects_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ux_project_lfs_objects_project_oid ON project_lfs_objects USING btree (project_id, oid);

CREATE TABLE project_members (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    user_id bigint NOT NULL,
    role text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_members_pkey PRIMARY KEY (id),
    CONSTRAINT project_members_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    CONSTRAINT project_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_members_user ON project_members USING btree (user_id);

CREATE UNIQUE INDEX ux_project_members_project_user ON project_members USING btree (project_id, user_id);

CREATE TABLE project_merge_request_approval_rules (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    name text NOT NULL,
    target_branch text NOT NULL,
    approvals_required integer CONSTRAINT project_merge_request_approval_rule_approvals_required_not_null NOT NULL,
    eligible_user_ids text NOT NULL,
    code_owner integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_merge_request_approval_rules_pkey PRIMARY KEY (id),
    CONSTRAINT project_merge_request_approval_rules_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_mr_approval_rules_code_owner ON project_merge_request_approval_rules USING btree (code_owner);

CREATE INDEX ix_project_mr_approval_rules_project_branch ON project_merge_request_approval_rules USING btree (project_id, target_branch);
