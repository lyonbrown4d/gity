CREATE INDEX IF NOT EXISTS "ix_project_pipeline_jobs_pipeline_stage_order" ON "project_pipeline_jobs" ("pipeline_id", "stage", "sort_order");
