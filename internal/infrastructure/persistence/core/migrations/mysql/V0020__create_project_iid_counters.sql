CREATE TABLE IF NOT EXISTS `project_iid_counters` (
    `project_id` BIGINT NOT NULL,
    `counter_name` VARCHAR(64) NOT NULL,
    `current_value` BIGINT NOT NULL DEFAULT 0,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    PRIMARY KEY (`project_id`, `counter_name`),
    CONSTRAINT `fk_project_iid_counters_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
