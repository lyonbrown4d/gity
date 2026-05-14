CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_lfs_objects_project_oid" ON "project_lfs_objects" ("project_id", "oid");
