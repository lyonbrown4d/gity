CREATE TABLE IF NOT EXISTS "project_issue_attachments" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_issue_id" BIGINT NOT NULL,
    "uploaded_by_user_id" BIGINT NOT NULL,
    "file_name" TEXT NOT NULL,
    "content_type" TEXT,
    "byte_size" BIGINT NOT NULL,
    "storage_key" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("uploaded_by_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);
