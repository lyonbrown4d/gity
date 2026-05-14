CREATE INDEX IF NOT EXISTS "ix_project_pipelines_project_ref" ON "project_pipelines" ("project_id", "ref_name");
