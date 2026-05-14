CREATE TABLE IF NOT EXISTS "project_pipeline_jobs" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "pipeline_id" BIGINT NOT NULL,
    "project_job_id" BIGINT NOT NULL,
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
