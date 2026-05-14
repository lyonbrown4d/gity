CREATE TABLE IF NOT EXISTS `project_pipelines` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `iid` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `source` VARCHAR(64) NOT NULL,
    `ref_name` VARCHAR(255),
    `commit_sha` VARCHAR(255),
    `status` VARCHAR(64) NOT NULL,
    `config_source` VARCHAR(64) NOT NULL,
    `config_content` TEXT NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    `started_at` DATETIME(6) NOT NULL,
    `finished_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_pipelines_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
