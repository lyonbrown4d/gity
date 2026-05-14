CREATE INDEX IF NOT EXISTS "ix_project_runners_active_contact" ON "project_runners" ("active", "last_contact_at");
