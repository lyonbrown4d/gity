CREATE TABLE IF NOT EXISTS `project_members` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `user_id` BIGINT NOT NULL,
    `role` VARCHAR(64) NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_members_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_members_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
    UNIQUE KEY `ux_project_members_project_user` (`project_id`, `user_id`),
    KEY `ix_project_members_user` (`user_id`)
);
