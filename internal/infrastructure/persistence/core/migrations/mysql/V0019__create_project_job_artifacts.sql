CREATE TABLE IF NOT EXISTS `project_job_artifacts` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `project_job_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `file_name` VARCHAR(255) NOT NULL,
    `file_path` VARCHAR(512) NOT NULL,
    `content_type` VARCHAR(255) NOT NULL,
    `byte_size` BIGINT NOT NULL,
    `storage_key` VARCHAR(512) NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_job_artifacts_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_job_artifacts_project_job_id` FOREIGN KEY (`project_job_id`) REFERENCES `project_jobs` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
