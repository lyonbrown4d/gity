CREATE TABLE IF NOT EXISTS "project_deploy_keys" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "fingerprint" TEXT NOT NULL,
    "public_key" TEXT NOT NULL,
    "can_push" INTEGER NOT NULL,
    "created_by_user_id" INTEGER NOT NULL,
    "last_used_at" TIMESTAMP NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("created_by_user_id") REFERENCES "users" ("id")
);

CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_deploy_keys_project_fingerprint" ON "project_deploy_keys" ("project_id", "fingerprint");
CREATE INDEX IF NOT EXISTS "ix_project_deploy_keys_created_by" ON "project_deploy_keys" ("created_by_user_id");
