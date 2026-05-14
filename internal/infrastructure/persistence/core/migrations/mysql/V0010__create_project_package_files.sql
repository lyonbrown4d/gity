CREATE TABLE IF NOT EXISTS `project_package_files` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_package_version_id` BIGINT NOT NULL,
    `file_name` VARCHAR(255) NOT NULL,
    `file_path` VARCHAR(512) NOT NULL,
    `content_type` VARCHAR(255),
    `byte_size` BIGINT NOT NULL,
    `storage_key` VARCHAR(512) NOT NULL UNIQUE,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_package_files_project_package_version_id` FOREIGN KEY (`project_package_version_id`) REFERENCES `project_package_versions` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
