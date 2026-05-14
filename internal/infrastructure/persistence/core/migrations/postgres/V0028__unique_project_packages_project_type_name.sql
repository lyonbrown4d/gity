CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_packages_project_type_name" ON "project_packages" ("project_id", "type", "name");
