CREATE INDEX IF NOT EXISTS "ix_project_merge_requests_project_state" ON "project_merge_requests" ("project_id", "state");
