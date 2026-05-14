CREATE TABLE IF NOT EXISTS "project_lfs_locks" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "owner_user_id" BIGINT NOT NULL,
    "path" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("owner_user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
