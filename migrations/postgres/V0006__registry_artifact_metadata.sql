ALTER TABLE project_package_files ADD COLUMN sha256 text NOT NULL DEFAULT '';
ALTER TABLE project_package_files ADD COLUMN created_by_user_id bigint;
ALTER TABLE project_package_files ADD COLUMN source text NOT NULL DEFAULT 'manual';
ALTER TABLE project_package_files ADD COLUMN pipeline_id bigint;
ALTER TABLE project_package_files ADD COLUMN job_id bigint;
ALTER TABLE project_package_files ADD COLUMN registry_metadata text;

CREATE INDEX ix_project_package_files_sha256 ON project_package_files USING btree (sha256);
CREATE INDEX ix_project_package_files_created_by ON project_package_files USING btree (created_by_user_id);

CREATE TABLE artifact_storage_deletions (
    id bigint NOT NULL PRIMARY KEY,
    storage_key text NOT NULL UNIQUE,
    attempts bigint NOT NULL DEFAULT 0,
    created_at timestamp without time zone NOT NULL
);

CREATE TABLE project_npm_dist_tags (
    id bigint NOT NULL PRIMARY KEY,
    project_package_id bigint NOT NULL,
    tag text NOT NULL,
    version text NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    CONSTRAINT project_npm_dist_tags_package_tag_key UNIQUE (project_package_id, tag),
    CONSTRAINT project_npm_dist_tags_package_fkey FOREIGN KEY (project_package_id) REFERENCES project_packages(id) ON DELETE CASCADE
);
