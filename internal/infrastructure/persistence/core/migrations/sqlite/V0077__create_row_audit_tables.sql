CREATE TABLE IF NOT EXISTS "audit_revisions" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL,
    "actor" TEXT NULL,
    "tenant" TEXT NULL,
    "reason" TEXT NULL,
    "metadata" TEXT NULL
);

CREATE INDEX IF NOT EXISTS "ix_audit_revisions_created_at" ON "audit_revisions" ("created_at");
CREATE INDEX IF NOT EXISTS "ix_audit_revisions_actor" ON "audit_revisions" ("actor");

CREATE TABLE IF NOT EXISTS "project_branch_protection_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_branch_protection_id" INTEGER NOT NULL,
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
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX IF NOT EXISTS "ix_project_branch_protection_audits_revision" ON "project_branch_protection_audits" ("revision_id");
CREATE INDEX IF NOT EXISTS "ix_project_branch_protection_audits_project" ON "project_branch_protection_audits" ("project_id");
CREATE INDEX IF NOT EXISTS "ix_project_branch_protection_audits_entity" ON "project_branch_protection_audits" ("project_branch_protection_id");

CREATE TABLE IF NOT EXISTS "project_member_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_member_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "role" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX IF NOT EXISTS "ix_project_member_audits_revision" ON "project_member_audits" ("revision_id");
CREATE INDEX IF NOT EXISTS "ix_project_member_audits_project" ON "project_member_audits" ("project_id");
CREATE INDEX IF NOT EXISTS "ix_project_member_audits_entity" ON "project_member_audits" ("project_member_id");
CREATE INDEX IF NOT EXISTS "ix_project_member_audits_user" ON "project_member_audits" ("user_id");

CREATE TABLE IF NOT EXISTS "project_merge_request_approval_rule_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_merge_request_approval_rule_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "target_branch" TEXT NOT NULL,
    "approvals_required" INTEGER NOT NULL,
    "eligible_user_ids" TEXT NOT NULL,
    "code_owner" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX IF NOT EXISTS "ix_project_mr_approval_rule_audits_revision" ON "project_merge_request_approval_rule_audits" ("revision_id");
CREATE INDEX IF NOT EXISTS "ix_project_mr_approval_rule_audits_project" ON "project_merge_request_approval_rule_audits" ("project_id");
CREATE INDEX IF NOT EXISTS "ix_project_mr_approval_rule_audits_entity" ON "project_merge_request_approval_rule_audits" ("project_merge_request_approval_rule_id");

CREATE TABLE IF NOT EXISTS "project_ci_variable_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_ci_variable_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "key" TEXT NOT NULL,
    "masked" INTEGER NOT NULL,
    "protected" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX IF NOT EXISTS "ix_project_ci_variable_audits_revision" ON "project_ci_variable_audits" ("revision_id");
CREATE INDEX IF NOT EXISTS "ix_project_ci_variable_audits_project" ON "project_ci_variable_audits" ("project_id");
CREATE INDEX IF NOT EXISTS "ix_project_ci_variable_audits_entity" ON "project_ci_variable_audits" ("project_ci_variable_id");
CREATE INDEX IF NOT EXISTS "ix_project_ci_variable_audits_key" ON "project_ci_variable_audits" ("key");

CREATE TABLE IF NOT EXISTS "project_deploy_key_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_deploy_key_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "fingerprint" TEXT NOT NULL,
    "can_push" INTEGER NOT NULL,
    "created_by_user_id" INTEGER NOT NULL,
    "last_used_at" TIMESTAMP NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX IF NOT EXISTS "ix_project_deploy_key_audits_revision" ON "project_deploy_key_audits" ("revision_id");
CREATE INDEX IF NOT EXISTS "ix_project_deploy_key_audits_project" ON "project_deploy_key_audits" ("project_id");
CREATE INDEX IF NOT EXISTS "ix_project_deploy_key_audits_entity" ON "project_deploy_key_audits" ("project_deploy_key_id");
CREATE INDEX IF NOT EXISTS "ix_project_deploy_key_audits_fingerprint" ON "project_deploy_key_audits" ("fingerprint");
