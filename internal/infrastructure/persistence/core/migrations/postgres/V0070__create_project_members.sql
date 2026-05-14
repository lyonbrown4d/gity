CREATE TABLE IF NOT EXISTS "project_members" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "user_id" BIGINT NOT NULL,
    "role" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_members_project_user" ON "project_members" ("project_id", "user_id");
CREATE INDEX IF NOT EXISTS "ix_project_members_user" ON "project_members" ("user_id");
