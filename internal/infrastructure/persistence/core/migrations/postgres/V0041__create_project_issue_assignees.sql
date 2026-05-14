CREATE TABLE IF NOT EXISTS "project_issue_assignees" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_issue_id" BIGINT NOT NULL,
    "user_id" BIGINT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_issue_id", "user_id"),
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
