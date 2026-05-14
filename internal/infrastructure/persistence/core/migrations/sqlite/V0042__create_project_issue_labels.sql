CREATE TABLE IF NOT EXISTS "project_issue_labels" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_issue_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "color" TEXT,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_issue_id", "name"),
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE
);
