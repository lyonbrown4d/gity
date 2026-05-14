CREATE INDEX IF NOT EXISTS "ix_project_issue_comments_issue_created" ON "project_issue_comments" ("project_issue_id", "created_at");
