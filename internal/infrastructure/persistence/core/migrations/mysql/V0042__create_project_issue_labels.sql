CREATE TABLE IF NOT EXISTS `project_issue_labels` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_issue_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `color` VARCHAR(64),
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    UNIQUE (`project_issue_id`, `name`),
    CONSTRAINT `fk_project_issue_labels_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
