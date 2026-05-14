CREATE TABLE IF NOT EXISTS "project_merge_request_approval_rules" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "name" TEXT NOT NULL,
    "target_branch" TEXT NOT NULL,
    "approvals_required" INTEGER NOT NULL,
    "eligible_user_ids" TEXT NOT NULL,
    "code_owner" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS "ix_project_mr_approval_rules_project_branch" ON "project_merge_request_approval_rules" ("project_id", "target_branch");
CREATE INDEX IF NOT EXISTS "ix_project_mr_approval_rules_code_owner" ON "project_merge_request_approval_rules" ("code_owner");
