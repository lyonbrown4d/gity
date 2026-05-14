CREATE TABLE IF NOT EXISTS `project_releases` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `tag_name` VARCHAR(255) NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `description` TEXT NOT NULL,
    `created_by_user_id` BIGINT NOT NULL,
    `released_at` TIMESTAMP(6) NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_releases_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_releases_created_by` FOREIGN KEY (`created_by_user_id`) REFERENCES `users` (`id`),
    UNIQUE KEY `ux_project_releases_project_tag` (`project_id`, `tag_name`),
    KEY `ix_project_releases_project_released` (`project_id`, `released_at`),
    KEY `ix_project_releases_created_by` (`created_by_user_id`)
);
