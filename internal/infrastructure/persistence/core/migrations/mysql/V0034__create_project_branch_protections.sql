CREATE TABLE IF NOT EXISTS `project_branch_protections` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `branch_name` VARCHAR(255) NOT NULL,
    `rule_type` VARCHAR(64) NOT NULL,
    `push_access_level` VARCHAR(64) NOT NULL,
    `merge_access_level` VARCHAR(64) NOT NULL,
    `require_merge_request` INTEGER NOT NULL,
    `require_pipeline_success` INTEGER NOT NULL,
    `allow_force_push` INTEGER NOT NULL,
    `allow_delete` INTEGER NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    UNIQUE (`project_id`, `branch_name`),
    CONSTRAINT `fk_project_branch_protections_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
