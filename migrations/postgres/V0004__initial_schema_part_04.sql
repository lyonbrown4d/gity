-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE project_job_artifacts (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    project_job_id bigint NOT NULL,
    name text NOT NULL,
    file_name text NOT NULL,
    file_path text NOT NULL,
    content_type text NOT NULL,
    byte_size bigint NOT NULL,
    storage_key text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_job_artifacts_pkey PRIMARY KEY (id),
    CONSTRAINT project_job_artifacts_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    CONSTRAINT project_job_artifacts_project_job_id_fkey FOREIGN KEY (project_job_id) REFERENCES project_jobs(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_job_artifacts_job ON project_job_artifacts USING btree (project_job_id);

CREATE TABLE project_job_logs (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    project_job_id bigint NOT NULL,
    attempt integer NOT NULL,
    exit_code integer NOT NULL,
    output text NOT NULL,
    output_truncated integer NOT NULL,
    duration_millis bigint NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_job_logs_pkey PRIMARY KEY (id),
    CONSTRAINT project_job_logs_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    CONSTRAINT project_job_logs_project_job_id_fkey FOREIGN KEY (project_job_id) REFERENCES project_jobs(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ux_project_job_logs_job_attempt ON project_job_logs USING btree (project_job_id, attempt);

CREATE TABLE project_merge_request_approvals (
    id bigint NOT NULL,
    merge_request_id bigint NOT NULL,
    user_id bigint NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_merge_request_approvals_merge_request_id_user_id_key UNIQUE (merge_request_id, user_id),
    CONSTRAINT project_merge_request_approvals_pkey PRIMARY KEY (id),
    CONSTRAINT project_merge_request_approvals_merge_request_id_fkey FOREIGN KEY (merge_request_id) REFERENCES project_merge_requests(id) ON DELETE CASCADE,
    CONSTRAINT project_merge_request_approvals_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE project_merge_request_comments (
    id bigint NOT NULL,
    merge_request_id bigint NOT NULL,
    author_user_id bigint NOT NULL,
    body text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_merge_request_comments_pkey PRIMARY KEY (id),
    CONSTRAINT project_merge_request_comments_author_user_id_fkey FOREIGN KEY (author_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT project_merge_request_comments_merge_request_id_fkey FOREIGN KEY (merge_request_id) REFERENCES project_merge_requests(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_merge_request_comments_mr_created ON project_merge_request_comments USING btree (merge_request_id, created_at);

CREATE TABLE project_merge_request_participants (
    id bigint NOT NULL,
    merge_request_id bigint NOT NULL,
    user_id bigint NOT NULL,
    role text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_merge_request_partici_merge_request_id_user_id_role_key UNIQUE (merge_request_id, user_id, role),
    CONSTRAINT project_merge_request_participants_pkey PRIMARY KEY (id),
    CONSTRAINT project_merge_request_participants_merge_request_id_fkey FOREIGN KEY (merge_request_id) REFERENCES project_merge_requests(id) ON DELETE CASCADE,
    CONSTRAINT project_merge_request_participants_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_merge_request_participants_user ON project_merge_request_participants USING btree (user_id);

CREATE TABLE project_package_versions (
    id bigint NOT NULL,
    project_package_id bigint NOT NULL,
    version text NOT NULL,
    status text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_package_versions_pkey PRIMARY KEY (id),
    CONSTRAINT project_package_versions_project_package_id_fkey FOREIGN KEY (project_package_id) REFERENCES project_packages(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_package_versions_package_status ON project_package_versions USING btree (project_package_id, status);

CREATE UNIQUE INDEX ux_project_package_versions_package_version ON project_package_versions USING btree (project_package_id, version);

CREATE TABLE project_pipeline_jobs (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    pipeline_id bigint NOT NULL,
    project_job_id bigint NOT NULL,
    name text NOT NULL,
    stage text NOT NULL,
    needs text,
    image text,
    script text NOT NULL,
    artifacts text,
    sort_order integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_pipeline_jobs_pkey PRIMARY KEY (id),
    CONSTRAINT project_pipeline_jobs_pipeline_id_fkey FOREIGN KEY (pipeline_id) REFERENCES project_pipelines(id) ON DELETE CASCADE,
    CONSTRAINT project_pipeline_jobs_project_id_fkey FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    CONSTRAINT project_pipeline_jobs_project_job_id_fkey FOREIGN KEY (project_job_id) REFERENCES project_jobs(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_pipeline_jobs_pipeline_stage_order ON project_pipeline_jobs USING btree (pipeline_id, stage, sort_order);

CREATE TABLE project_release_links (
    id bigint NOT NULL,
    project_release_id bigint NOT NULL,
    name text NOT NULL,
    url text NOT NULL,
    link_type text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_release_links_pkey PRIMARY KEY (id),
    CONSTRAINT project_release_links_project_release_id_fkey FOREIGN KEY (project_release_id) REFERENCES project_releases(id) ON DELETE CASCADE
);

CREATE INDEX ix_project_release_links_release ON project_release_links USING btree (project_release_id);

CREATE INDEX ix_project_release_links_type ON project_release_links USING btree (link_type);

CREATE TABLE project_package_files (
    id bigint NOT NULL,
    project_package_version_id bigint NOT NULL,
    file_name text NOT NULL,
    file_path text NOT NULL,
    content_type text,
    byte_size bigint NOT NULL,
    storage_key text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_package_files_pkey PRIMARY KEY (id),
    CONSTRAINT project_package_files_storage_key_key UNIQUE (storage_key),
    CONSTRAINT project_package_files_project_package_version_id_fkey FOREIGN KEY (project_package_version_id) REFERENCES project_package_versions(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ux_project_package_files_version_path ON project_package_files USING btree (project_package_version_id, file_path);
