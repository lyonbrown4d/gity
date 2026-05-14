CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_issues_project_iid" ON "project_issues" ("project_id", "iid");
