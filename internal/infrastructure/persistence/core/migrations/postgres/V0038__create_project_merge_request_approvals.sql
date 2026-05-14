CREATE TABLE IF NOT EXISTS "project_merge_request_approvals" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "merge_request_id" BIGINT NOT NULL,
    "user_id" BIGINT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("merge_request_id", "user_id"),
    FOREIGN KEY ("merge_request_id") REFERENCES "project_merge_requests" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
