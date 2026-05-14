CREATE INDEX IF NOT EXISTS "ix_project_pipelines_project_status" ON "project_pipelines" ("project_id", "status");
