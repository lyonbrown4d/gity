CREATE TABLE IF NOT EXISTS "project_job_artifacts" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "project_job_id" BIGINT NOT NULL,
    "name" TEXT NOT NULL,
    "file_name" TEXT NOT NULL,
    "file_path" TEXT NOT NULL,
    "content_type" TEXT NOT NULL,
    "byte_size" BIGINT NOT NULL,
    "storage_key" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("project_job_id") REFERENCES "project_jobs" ("id") ON DELETE CASCADE
);
