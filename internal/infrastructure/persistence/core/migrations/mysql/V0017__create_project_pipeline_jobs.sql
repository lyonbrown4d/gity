CREATE TABLE IF NOT EXISTS `project_pipeline_jobs` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `pipeline_id` BIGINT NOT NULL,
    `project_job_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `stage` VARCHAR(255) NOT NULL,
    `needs` TEXT,
    `image` VARCHAR(255),
    `script` TEXT NOT NULL,
    `artifacts` TEXT,
    `sort_order` INTEGER NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_pipeline_jobs_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_pipeline_jobs_pipeline_id` FOREIGN KEY (`pipeline_id`) REFERENCES `project_pipelines` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_pipeline_jobs_project_job_id` FOREIGN KEY (`project_job_id`) REFERENCES `project_jobs` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
