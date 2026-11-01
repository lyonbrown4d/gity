-- Shared component metadata. Project IDs are application-generated identities.

CREATE TABLE "git_repository_placements" (
  "project_id" BIGINT NOT NULL PRIMARY KEY,
  "transition_target" TEXT NULL,
  "primary_node" TEXT NOT NULL,
  "epoch" BIGINT NOT NULL,
  "generation" BIGINT NOT NULL,
  "pending_writes" BIGINT NOT NULL,
  "state" TEXT NOT NULL,
  "replicas_json" TEXT NOT NULL,
  "version" BIGINT NOT NULL,
  "last_error" TEXT NULL,
  "updated_at" TIMESTAMP NOT NULL,
  CONSTRAINT "ck_git_repository_placements_project" CHECK ("project_id" > 0),
  CONSTRAINT "ck_git_placement_pending" CHECK ("pending_writes" >= 0)
);

CREATE TABLE "project_registry_settings" (
  "project_id" BIGINT NOT NULL PRIMARY KEY,
  "mode" TEXT NOT NULL,
  "mirror_alias" TEXT NULL,
  "mirror_repository_prefix" TEXT NOT NULL,
  "updated_by" BIGINT NULL,
  "updated_at" TIMESTAMP NOT NULL,
  CONSTRAINT "ck_project_registry_settings_project" CHECK ("project_id" > 0)
);
