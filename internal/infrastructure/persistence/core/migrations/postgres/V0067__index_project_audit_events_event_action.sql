CREATE INDEX IF NOT EXISTS "ix_project_audit_events_event_action" ON "project_audit_events" ("event_name", "action");
