-- Shared component metadata. Project IDs are application-generated identities.

CREATE TABLE `git_repository_placements` (
  `project_id` BIGINT NOT NULL PRIMARY KEY,
  `transition_target` VARCHAR(255) NULL,
  `primary_node` VARCHAR(255) NOT NULL,
  `epoch` BIGINT NOT NULL,
  `generation` BIGINT NOT NULL,
  `pending_writes` BIGINT NOT NULL,
  `state` VARCHAR(255) NOT NULL,
  `replicas_json` TEXT NOT NULL,
  `version` BIGINT NOT NULL,
  `last_error` VARCHAR(255) NULL,
  `updated_at` DATETIME(6) NOT NULL,
  CONSTRAINT `ck_git_repository_placements_project` CHECK (`project_id` > 0),
  CONSTRAINT `ck_git_placement_pending` CHECK (`pending_writes` >= 0)
);

CREATE TABLE `project_registry_settings` (
  `project_id` BIGINT NOT NULL PRIMARY KEY,
  `mode` VARCHAR(255) NOT NULL,
  `mirror_alias` VARCHAR(255) NULL,
  `mirror_repository_prefix` VARCHAR(255) NOT NULL,
  `updated_by` BIGINT NULL,
  `updated_at` DATETIME(6) NOT NULL,
  CONSTRAINT `ck_project_registry_settings_project` CHECK (`project_id` > 0)
);
