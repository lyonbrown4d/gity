CREATE TABLE IF NOT EXISTS "project_job_logs" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "project_job_id" BIGINT NOT NULL,
    "attempt" INTEGER NOT NULL,
    "exit_code" INTEGER NOT NULL,
    "output" TEXT NOT NULL,
    "output_truncated" INTEGER NOT NULL,
    "duration_millis" BIGINT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("project_job_id") REFERENCES "project_jobs" ("id") ON DELETE CASCADE
);
