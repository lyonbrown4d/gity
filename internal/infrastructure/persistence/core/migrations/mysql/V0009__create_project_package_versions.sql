CREATE TABLE IF NOT EXISTS `project_package_versions` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_package_id` BIGINT NOT NULL,
    `version` VARCHAR(255) NOT NULL,
    `status` VARCHAR(64) NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_package_versions_project_package_id` FOREIGN KEY (`project_package_id`) REFERENCES `project_packages` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
