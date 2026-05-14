CREATE TABLE IF NOT EXISTS `projects` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `organization_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `path_key` VARCHAR(255) NOT NULL,
    `full_path` VARCHAR(512) NOT NULL UNIQUE,
    `visibility` VARCHAR(64) NOT NULL,
    `description` TEXT,
    `default_branch` VARCHAR(255) NOT NULL,
    `status` VARCHAR(64) NOT NULL,
    `deleted_at` DATETIME(6),
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_projects_organization_id` FOREIGN KEY (`organization_id`) REFERENCES `organizations` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
