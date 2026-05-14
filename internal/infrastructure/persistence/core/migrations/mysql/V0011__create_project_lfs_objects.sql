CREATE TABLE IF NOT EXISTS `project_lfs_objects` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `oid` VARCHAR(255) NOT NULL,
    `byte_size` BIGINT NOT NULL,
    `storage_key` VARCHAR(512) NOT NULL UNIQUE,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_lfs_objects_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
