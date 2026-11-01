-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE project_merge_requests (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    iid bigint NOT NULL,
    author_user_id bigint NOT NULL,
    title text NOT NULL,
    description text,
    state text NOT NULL,
    source_branch text NOT NULL,
    target_branch text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_merge_requests_pkey PRIMARY KEY (id),
    CONSTRAINT project_merge_requests_author_user_id_fkey FOREIGN KEY (author_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT project_merge_requests_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_merge_requests_author ON project_merge_requests USING btree (author_user_id);

CREATE INDEX ix_project_merge_requests_branches ON project_merge_requests USING btree (project_id, source_branch, target_branch);

CREATE INDEX ix_project_merge_requests_project_state ON project_merge_requests USING btree (project_id, state);

CREATE UNIQUE INDEX ux_project_merge_requests_project_iid ON project_merge_requests USING btree (project_id, iid);

CREATE TABLE project_packages (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    type text NOT NULL,
    name text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_packages_pkey PRIMARY KEY (id),
    CONSTRAINT project_packages_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ux_project_packages_project_type_name ON project_packages USING btree (project_id, type, name);

CREATE TABLE project_pipelines (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    iid bigint NOT NULL,
    name text NOT NULL,
    source text NOT NULL,
    ref_name text,
    commit_sha text,
    status text NOT NULL,
    config_source text NOT NULL,
    config_content text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    started_at timestamp without time zone NOT NULL,
    finished_at timestamp without time zone NOT NULL,
    CONSTRAINT project_pipelines_pkey PRIMARY KEY (id),
    CONSTRAINT project_pipelines_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_pipelines_project_ref ON project_pipelines USING btree (project_id, ref_name);

CREATE INDEX ix_project_pipelines_project_status ON project_pipelines USING btree (project_id, status);

CREATE UNIQUE INDEX ux_project_pipelines_project_iid ON project_pipelines USING btree (project_id, iid);

CREATE TABLE project_releases (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    tag_name text NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    created_by_user_id bigint NOT NULL,
    released_at timestamp without time zone NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_releases_pkey PRIMARY KEY (id),
    CONSTRAINT project_releases_created_by_user_id_fkey FOREIGN KEY (created_by_user_id) REFERENCES users(id),
    CONSTRAINT project_releases_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_releases_created_by ON project_releases USING btree (created_by_user_id);

CREATE INDEX ix_project_releases_project_released ON project_releases USING btree (project_id, released_at);

CREATE UNIQUE INDEX ux_project_releases_project_tag ON project_releases USING btree (project_id, tag_name);

CREATE TABLE project_runners (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    name text NOT NULL,
    description text,
    tags text,
    token text NOT NULL,
    status text NOT NULL,
    active integer NOT NULL,
    last_contact_at timestamp without time zone NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_runners_pkey PRIMARY KEY (id),
    CONSTRAINT project_runners_token_key UNIQUE (token),
    CONSTRAINT project_runners_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_runners_active_contact ON project_runners USING btree (active, last_contact_at);

CREATE INDEX ix_project_runners_project_status ON project_runners USING btree (project_id, status);

CREATE TABLE project_todos (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    organization_id bigint NOT NULL,
    project_id bigint NOT NULL,
    kind text NOT NULL,
    state text NOT NULL,
    target_type text NOT NULL,
    target_id text NOT NULL,
    title text NOT NULL,
    summary text NOT NULL,
    action_url text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    done_at timestamp without time zone,
    CONSTRAINT project_todos_pkey PRIMARY KEY (id),
    CONSTRAINT project_todos_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT project_todos_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    CONSTRAINT project_todos_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_todos_organization_user ON project_todos USING btree (organization_id, user_id);

CREATE INDEX ix_project_todos_project_user ON project_todos USING btree (project_id, user_id);

CREATE INDEX ix_project_todos_user_state_created ON project_todos USING btree (user_id, state, created_at);

CREATE UNIQUE INDEX ux_project_todos_user_kind_target ON project_todos USING btree (user_id, kind, target_type, target_id);

CREATE TABLE project_wiki_pages (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    slug text NOT NULL,
    title text NOT NULL,
    content text,
    format text NOT NULL,
    author_user_id bigint NOT NULL,
    last_edited_by_user_id bigint NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_wiki_pages_pkey PRIMARY KEY (id),
    CONSTRAINT project_wiki_pages_project_id_slug_key UNIQUE (project_id, slug),
    CONSTRAINT project_wiki_pages_author_user_id_fkey FOREIGN KEY (author_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT project_wiki_pages_last_edited_by_user_id_fkey FOREIGN KEY (last_edited_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT project_wiki_pages_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_wiki_pages_project_format ON project_wiki_pages USING btree (project_id, format);

CREATE TABLE project_issue_assignees (
    id bigint NOT NULL,
    project_issue_id bigint NOT NULL,
    user_id bigint NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_issue_assignees_pkey PRIMARY KEY (id),
    CONSTRAINT project_issue_assignees_project_issue_id_user_id_key UNIQUE (project_issue_id, user_id),
    CONSTRAINT project_issue_assignees_project_issue_id_fkey FOREIGN KEY (project_issue_id) REFERENCES project_issues(id) ON DELETE CASCADE,
    CONSTRAINT project_issue_assignees_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_issue_assignees_user ON project_issue_assignees USING btree (user_id);

CREATE TABLE project_issue_attachments (
    id bigint NOT NULL,
    project_issue_id bigint NOT NULL,
    uploaded_by_user_id bigint NOT NULL,
    file_name text NOT NULL,
    content_type text,
    byte_size bigint NOT NULL,
    storage_key text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_issue_attachments_pkey PRIMARY KEY (id),
    CONSTRAINT project_issue_attachments_storage_key_key UNIQUE (storage_key),
    CONSTRAINT project_issue_attachments_project_issue_id_fkey FOREIGN KEY (project_issue_id) REFERENCES project_issues(id) ON DELETE CASCADE,
    CONSTRAINT project_issue_attachments_uploaded_by_user_id_fkey FOREIGN KEY (uploaded_by_user_id) REFERENCES users(id) ON DELETE RESTRICT
);

CREATE TABLE project_issue_comments (
    id bigint NOT NULL,
    project_issue_id bigint NOT NULL,
    author_user_id bigint NOT NULL,
    body text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_issue_comments_pkey PRIMARY KEY (id),
    CONSTRAINT project_issue_comments_author_user_id_fkey FOREIGN KEY (author_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT project_issue_comments_project_issue_id_fkey FOREIGN KEY (project_issue_id) REFERENCES project_issues(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_issue_comments_issue_created ON project_issue_comments USING btree (project_issue_id, created_at);

CREATE TABLE project_issue_labels (
    id bigint NOT NULL,
    project_issue_id bigint NOT NULL,
    name text NOT NULL,
    color text,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_issue_labels_pkey PRIMARY KEY (id),
    CONSTRAINT project_issue_labels_project_issue_id_name_key UNIQUE (project_issue_id, name),
    CONSTRAINT project_issue_labels_project_issue_id_fkey FOREIGN KEY (project_issue_id) REFERENCES project_issues(id) ON DELETE CASCADE
);
