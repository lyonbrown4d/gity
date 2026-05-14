CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_package_versions_package_version" ON "project_package_versions" ("project_package_id", "version");
