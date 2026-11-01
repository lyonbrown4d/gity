ALTER TABLE project_package_files ADD COLUMN sha256 varchar(64) NOT NULL DEFAULT '';
ALTER TABLE project_package_files ADD COLUMN created_by_user_id bigint NULL;
ALTER TABLE project_package_files ADD COLUMN source varchar(32) NOT NULL DEFAULT 'manual';
ALTER TABLE project_package_files ADD COLUMN pipeline_id bigint NULL;
ALTER TABLE project_package_files ADD COLUMN job_id bigint NULL;
ALTER TABLE project_package_files ADD COLUMN registry_metadata longtext NULL;

CREATE INDEX ix_project_package_files_sha256 ON project_package_files (sha256);
CREATE INDEX ix_project_package_files_created_by ON project_package_files (created_by_user_id);

CREATE TABLE artifact_storage_deletions (
    id bigint NOT NULL PRIMARY KEY,
    storage_key varchar(512) NOT NULL UNIQUE,
    attempts bigint NOT NULL DEFAULT 0,
    created_at datetime(6) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE project_npm_dist_tags (
    id bigint NOT NULL PRIMARY KEY,
    project_package_id bigint NOT NULL,
    tag varchar(128) NOT NULL,
    version varchar(255) NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    UNIQUE KEY ux_project_npm_dist_tags_package_tag (project_package_id, tag),
    CONSTRAINT fk_project_npm_dist_tags_package FOREIGN KEY (project_package_id) REFERENCES project_packages (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
