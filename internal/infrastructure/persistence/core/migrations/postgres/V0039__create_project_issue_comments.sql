CREATE TABLE IF NOT EXISTS "project_issue_comments" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_issue_id" BIGINT NOT NULL,
    "author_user_id" BIGINT NOT NULL,
    "body" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);
