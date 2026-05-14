CREATE TABLE IF NOT EXISTS "project_audit_events" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "organization_id" INTEGER NOT NULL,
    "event_name" TEXT NOT NULL,
    "action" TEXT NOT NULL,
    "actor_user_id" INTEGER NOT NULL,
    "target_type" TEXT NOT NULL,
    "target_id" TEXT NOT NULL,
    "summary" TEXT NOT NULL,
    "payload" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE
);
