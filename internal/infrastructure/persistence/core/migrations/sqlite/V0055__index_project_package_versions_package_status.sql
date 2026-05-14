CREATE INDEX IF NOT EXISTS "ix_project_package_versions_package_status" ON "project_package_versions" ("project_package_id", "status");
