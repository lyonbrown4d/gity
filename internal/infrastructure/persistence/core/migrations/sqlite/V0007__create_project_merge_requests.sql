CREATE TABLE IF NOT EXISTS "project_merge_requests" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "iid" INTEGER NOT NULL,
    "author_user_id" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "state" TEXT NOT NULL,
    "source_branch" TEXT NOT NULL,
    "target_branch" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);
