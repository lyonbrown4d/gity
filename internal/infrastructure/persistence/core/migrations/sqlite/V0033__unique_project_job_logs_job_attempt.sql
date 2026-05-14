CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_job_logs_job_attempt" ON "project_job_logs" ("project_job_id", "attempt");
