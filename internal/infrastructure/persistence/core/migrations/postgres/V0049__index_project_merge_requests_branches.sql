CREATE INDEX IF NOT EXISTS "ix_project_merge_requests_branches" ON "project_merge_requests" ("project_id", "source_branch", "target_branch");
