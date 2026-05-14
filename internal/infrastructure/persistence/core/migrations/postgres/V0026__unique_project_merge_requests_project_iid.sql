CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_merge_requests_project_iid" ON "project_merge_requests" ("project_id", "iid");
