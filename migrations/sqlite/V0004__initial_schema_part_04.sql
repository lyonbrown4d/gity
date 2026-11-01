-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE "project_job_logs" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "project_job_id" INTEGER NOT NULL,
    "attempt" INTEGER NOT NULL,
    "exit_code" INTEGER NOT NULL,
    "output" TEXT NOT NULL,
    "output_truncated" INTEGER NOT NULL,
    "duration_millis" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("project_job_id") REFERENCES "project_jobs" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX "ux_project_job_logs_job_attempt" ON "project_job_logs" ("project_job_id", "attempt");

CREATE TABLE "project_merge_request_approvals" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "merge_request_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("merge_request_id", "user_id"),
    FOREIGN KEY ("merge_request_id") REFERENCES "project_merge_requests" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE TABLE "project_merge_request_comments" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "merge_request_id" INTEGER NOT NULL,
    "author_user_id" INTEGER NOT NULL,
    "body" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("merge_request_id") REFERENCES "project_merge_requests" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);

CREATE INDEX "ix_project_merge_request_comments_mr_created" ON "project_merge_request_comments" ("merge_request_id", "created_at");

CREATE TABLE "project_merge_request_participants" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "merge_request_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "role" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("merge_request_id", "user_id", "role"),
    FOREIGN KEY ("merge_request_id") REFERENCES "project_merge_requests" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_merge_request_participants_user" ON "project_merge_request_participants" ("user_id");

CREATE TABLE "project_package_versions" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_package_id" INTEGER NOT NULL,
    "version" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_package_id") REFERENCES "project_packages" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_package_versions_package_status" ON "project_package_versions" ("project_package_id", "status");

CREATE UNIQUE INDEX "ux_project_package_versions_package_version" ON "project_package_versions" ("project_package_id", "version");

CREATE TABLE "project_pipeline_jobs" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "pipeline_id" INTEGER NOT NULL,
    "project_job_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "stage" TEXT NOT NULL,
    "needs" TEXT,
    "image" TEXT,
    "script" TEXT NOT NULL,
    "artifacts" TEXT,
    "sort_order" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("pipeline_id") REFERENCES "project_pipelines" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("project_job_id") REFERENCES "project_jobs" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_pipeline_jobs_pipeline_stage_order" ON "project_pipeline_jobs" ("pipeline_id", "stage", "sort_order");

CREATE TABLE "project_release_links" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_release_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "url" TEXT NOT NULL,
    "link_type" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_release_id") REFERENCES "project_releases" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_release_links_release" ON "project_release_links" ("project_release_id");

CREATE INDEX "ix_project_release_links_type" ON "project_release_links" ("link_type");

CREATE TABLE "project_package_files" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_package_version_id" INTEGER NOT NULL,
    "file_name" TEXT NOT NULL,
    "file_path" TEXT NOT NULL,
    "content_type" TEXT,
    "byte_size" INTEGER NOT NULL,
    "storage_key" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_package_version_id") REFERENCES "project_package_versions" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX "ux_project_package_files_version_path" ON "project_package_files" ("project_package_version_id", "file_path");
