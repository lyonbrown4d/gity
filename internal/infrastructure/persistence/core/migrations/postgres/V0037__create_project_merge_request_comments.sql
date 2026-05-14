CREATE TABLE IF NOT EXISTS "project_merge_request_comments" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "merge_request_id" BIGINT NOT NULL,
    "author_user_id" BIGINT NOT NULL,
    "body" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("merge_request_id") REFERENCES "project_merge_requests" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);
