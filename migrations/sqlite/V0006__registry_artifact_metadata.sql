ALTER TABLE project_package_files ADD COLUMN sha256 TEXT NOT NULL DEFAULT '';
ALTER TABLE project_package_files ADD COLUMN created_by_user_id INTEGER;
ALTER TABLE project_package_files ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE project_package_files ADD COLUMN pipeline_id INTEGER;
ALTER TABLE project_package_files ADD COLUMN job_id INTEGER;
ALTER TABLE project_package_files ADD COLUMN registry_metadata TEXT;

CREATE INDEX ix_project_package_files_sha256 ON project_package_files (sha256);
CREATE INDEX ix_project_package_files_created_by ON project_package_files (created_by_user_id);

CREATE TABLE artifact_storage_deletions (
    id INTEGER NOT NULL PRIMARY KEY,
    storage_key TEXT NOT NULL UNIQUE,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE project_npm_dist_tags (
    id INTEGER NOT NULL PRIMARY KEY,
    project_package_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    version TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    UNIQUE (project_package_id, tag),
    FOREIGN KEY (project_package_id) REFERENCES project_packages (id) ON DELETE CASCADE
);
