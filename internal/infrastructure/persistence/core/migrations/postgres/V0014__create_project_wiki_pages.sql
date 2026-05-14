CREATE TABLE IF NOT EXISTS "project_wiki_pages" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "slug" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "content" TEXT,
    "format" TEXT NOT NULL,
    "author_user_id" BIGINT NOT NULL,
    "last_edited_by_user_id" BIGINT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_id", "slug"),
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT,
    FOREIGN KEY ("last_edited_by_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);
