CREATE TABLE IF NOT EXISTS `project_runners` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `description` TEXT,
    `tags` VARCHAR(255),
    `token` VARCHAR(255) NOT NULL UNIQUE,
    `status` VARCHAR(64) NOT NULL,
    `active` INTEGER NOT NULL,
    `last_contact_at` DATETIME(6) NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_runners_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
