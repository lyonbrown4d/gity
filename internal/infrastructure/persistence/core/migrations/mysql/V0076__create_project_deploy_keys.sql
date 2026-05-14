CREATE TABLE IF NOT EXISTS `project_deploy_keys` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `title` VARCHAR(255) NOT NULL,
    `fingerprint` VARCHAR(255) NOT NULL,
    `public_key` TEXT NOT NULL,
    `can_push` INTEGER NOT NULL,
    `created_by_user_id` BIGINT NOT NULL,
    `last_used_at` TIMESTAMP(6) NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_deploy_keys_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_deploy_keys_created_by` FOREIGN KEY (`created_by_user_id`) REFERENCES `users` (`id`),
    UNIQUE KEY `ux_project_deploy_keys_project_fingerprint` (`project_id`, `fingerprint`),
    KEY `ix_project_deploy_keys_created_by` (`created_by_user_id`)
);
