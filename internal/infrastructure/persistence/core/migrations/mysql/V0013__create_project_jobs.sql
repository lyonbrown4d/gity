CREATE TABLE IF NOT EXISTS `project_jobs` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `kind` VARCHAR(64) NOT NULL,
    `status` VARCHAR(64) NOT NULL,
    `payload` TEXT,
    `result` TEXT,
    `attempts` INTEGER NOT NULL,
    `max_attempts` INTEGER NOT NULL,
    `run_after` DATETIME(6) NOT NULL,
    `locked_by` VARCHAR(255),
    `locked_until` DATETIME(6) NOT NULL,
    `last_error` TEXT,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    `started_at` DATETIME(6) NOT NULL,
    `finished_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_jobs_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
