CREATE TABLE IF NOT EXISTS "project_access_tokens" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "kind" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "username" TEXT NOT NULL,
    "token" TEXT NOT NULL,
    "scopes" TEXT NOT NULL,
    "created_by_user_id" INTEGER NOT NULL,
    "expires_at" TIMESTAMP NULL,
    "revoked_at" TIMESTAMP NULL,
    "last_used_at" TIMESTAMP NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("created_by_user_id") REFERENCES "users" ("id")
);

CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_access_tokens_token" ON "project_access_tokens" ("token");
CREATE INDEX IF NOT EXISTS "ix_project_access_tokens_project_kind" ON "project_access_tokens" ("project_id", "kind");
CREATE INDEX IF NOT EXISTS "ix_project_access_tokens_username" ON "project_access_tokens" ("username");
CREATE INDEX IF NOT EXISTS "ix_project_access_tokens_created_by" ON "project_access_tokens" ("created_by_user_id");
