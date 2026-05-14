CREATE INDEX IF NOT EXISTS "ix_project_jobs_queue" ON "project_jobs" ("status", "run_after", "kind");
