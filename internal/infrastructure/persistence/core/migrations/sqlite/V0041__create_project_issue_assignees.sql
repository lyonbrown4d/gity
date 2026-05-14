CREATE TABLE IF NOT EXISTS "project_issue_assignees" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_issue_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_issue_id", "user_id"),
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
