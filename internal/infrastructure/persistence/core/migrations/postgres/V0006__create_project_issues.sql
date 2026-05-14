CREATE TABLE IF NOT EXISTS "project_issues" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "iid" BIGINT NOT NULL,
    "author_user_id" BIGINT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "state" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);
