CREATE TABLE IF NOT EXISTS "project_lfs_objects" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "oid" TEXT NOT NULL,
    "byte_size" BIGINT NOT NULL,
    "storage_key" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);
