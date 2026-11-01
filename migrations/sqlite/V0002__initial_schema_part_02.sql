-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE "project_access_tokens" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "kind" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "username" TEXT NOT NULL,
    "token" TEXT NOT NULL,
    "scopes" TEXT NOT NULL,
    "created_by_user_id" INTEGER NOT NULL,
    "expires_at" TIMESTAMP NULL,
    "revoked_at" TIMESTAMP NULL,
    "last_used_at" TIMESTAMP NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("created_by_user_id") REFERENCES "users" ("id")
);

CREATE INDEX "ix_project_access_tokens_created_by" ON "project_access_tokens" ("created_by_user_id");

CREATE INDEX "ix_project_access_tokens_project_kind" ON "project_access_tokens" ("project_id", "kind");

CREATE INDEX "ix_project_access_tokens_username" ON "project_access_tokens" ("username");

CREATE UNIQUE INDEX "ux_project_access_tokens_token" ON "project_access_tokens" ("token");

CREATE TABLE "project_audit_events" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "organization_id" INTEGER NOT NULL,
    "event_name" TEXT NOT NULL,
    "action" TEXT NOT NULL,
    "actor_user_id" INTEGER NOT NULL,
    "target_type" TEXT NOT NULL,
    "target_id" TEXT NOT NULL,
    "summary" TEXT NOT NULL,
    "payload" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_audit_events_actor" ON "project_audit_events" ("actor_user_id");

CREATE INDEX "ix_project_audit_events_event_action" ON "project_audit_events" ("event_name", "action");

CREATE INDEX "ix_project_audit_events_project_created" ON "project_audit_events" ("project_id", "created_at");

CREATE TABLE "project_branch_protections" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
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

CREATE TABLE "project_ci_variables" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "key" TEXT NOT NULL,
    "value" TEXT NOT NULL,
    "masked" INTEGER NOT NULL,
    "protected" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_ci_variables_flags" ON "project_ci_variables" ("masked", "protected");

CREATE UNIQUE INDEX "ux_project_ci_variables_project_key" ON "project_ci_variables" ("project_id", "key");

CREATE TABLE "project_deploy_keys" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "fingerprint" TEXT NOT NULL,
    "public_key" TEXT NOT NULL,
    "can_push" INTEGER NOT NULL,
    "created_by_user_id" INTEGER NOT NULL,
    "last_used_at" TIMESTAMP NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("created_by_user_id") REFERENCES "users" ("id")
);

CREATE INDEX "ix_project_deploy_keys_created_by" ON "project_deploy_keys" ("created_by_user_id");

CREATE UNIQUE INDEX "ux_project_deploy_keys_project_fingerprint" ON "project_deploy_keys" ("project_id", "fingerprint");

CREATE TABLE "project_iid_counters" (
    "project_id" INTEGER NOT NULL,
    "counter_name" TEXT NOT NULL,
    "current_value" INTEGER NOT NULL DEFAULT 0,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    PRIMARY KEY ("project_id", "counter_name"),
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE TABLE "project_issues" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "iid" INTEGER NOT NULL,
    "author_user_id" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "state" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);

CREATE INDEX "ix_project_issues_author" ON "project_issues" ("author_user_id");

CREATE INDEX "ix_project_issues_project_state" ON "project_issues" ("project_id", "state");

CREATE UNIQUE INDEX "ux_project_issues_project_iid" ON "project_issues" ("project_id", "iid");

CREATE TABLE "project_jobs" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "kind" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "payload" TEXT,
    "result" TEXT,
    "attempts" INTEGER NOT NULL,
    "max_attempts" INTEGER NOT NULL,
    "run_after" TIMESTAMP NOT NULL,
    "locked_by" TEXT,
    "locked_until" TIMESTAMP NOT NULL,
    "last_error" TEXT,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "started_at" TIMESTAMP NOT NULL,
    "finished_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_jobs_locked" ON "project_jobs" ("locked_by", "locked_until");

CREATE INDEX "ix_project_jobs_queue" ON "project_jobs" ("status", "run_after", "kind");

CREATE TABLE "project_lfs_locks" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "owner_user_id" INTEGER NOT NULL,
    "path" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("owner_user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_lfs_locks_owner" ON "project_lfs_locks" ("owner_user_id");

CREATE UNIQUE INDEX "ux_project_lfs_locks_project_path" ON "project_lfs_locks" ("project_id", "path");

CREATE TABLE "project_lfs_objects" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "oid" TEXT NOT NULL,
    "byte_size" INTEGER NOT NULL,
    "storage_key" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX "ux_project_lfs_objects_project_oid" ON "project_lfs_objects" ("project_id", "oid");

CREATE TABLE "project_members" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "role" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_members_user" ON "project_members" ("user_id");

CREATE UNIQUE INDEX "ux_project_members_project_user" ON "project_members" ("project_id", "user_id");

CREATE TABLE "project_merge_request_approval_rules" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "target_branch" TEXT NOT NULL,
    "approvals_required" INTEGER NOT NULL,
    "eligible_user_ids" TEXT NOT NULL,
    "code_owner" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_mr_approval_rules_code_owner" ON "project_merge_request_approval_rules" ("code_owner");

CREATE INDEX "ix_project_mr_approval_rules_project_branch" ON "project_merge_request_approval_rules" ("project_id", "target_branch");
