CREATE TABLE IF NOT EXISTS "project_branch_protections" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "branch_name" TEXT NOT NULL,
    "rule_type" TEXT NOT NULL,
    "push_access_level" TEXT NOT NULL,
    "merge_access_level" TEXT NOT NULL,
    "require_merge_request" INTEGER NOT NULL,
    "require_pipeline_success" INTEGER NOT NULL,
    "allow_force_push" INTEGER NOT NULL,
    "allow_delete" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_id", "branch_name"),
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);
