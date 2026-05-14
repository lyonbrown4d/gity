CREATE INDEX IF NOT EXISTS "ix_project_issues_project_state" ON "project_issues" ("project_id", "state");
