CREATE INDEX IF NOT EXISTS "ix_project_audit_events_project_created" ON "project_audit_events" ("project_id", "created_at");
