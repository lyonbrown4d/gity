CREATE INDEX IF NOT EXISTS "ix_project_jobs_locked" ON "project_jobs" ("locked_by", "locked_until");
