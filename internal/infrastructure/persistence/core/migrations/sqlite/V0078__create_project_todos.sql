CREATE TABLE IF NOT EXISTS "project_todos" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "user_id" INTEGER NOT NULL,
    "organization_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "kind" TEXT NOT NULL,
    "state" TEXT NOT NULL,
    "target_type" TEXT NOT NULL,
    "target_id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "summary" TEXT NOT NULL,
    "action_url" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "done_at" TIMESTAMP NULL,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_todos_user_kind_target" ON "project_todos" ("user_id", "kind", "target_type", "target_id");
CREATE INDEX IF NOT EXISTS "ix_project_todos_user_state_created" ON "project_todos" ("user_id", "state", "created_at");
CREATE INDEX IF NOT EXISTS "ix_project_todos_project_user" ON "project_todos" ("project_id", "user_id");
CREATE INDEX IF NOT EXISTS "ix_project_todos_organization_user" ON "project_todos" ("organization_id", "user_id");
