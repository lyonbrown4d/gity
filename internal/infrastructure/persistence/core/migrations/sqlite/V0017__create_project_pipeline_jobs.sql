CREATE TABLE IF NOT EXISTS "project_pipeline_jobs" (
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
