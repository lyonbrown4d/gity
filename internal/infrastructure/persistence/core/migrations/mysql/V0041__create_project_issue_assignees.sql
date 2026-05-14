CREATE TABLE IF NOT EXISTS `project_issue_assignees` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_issue_id` BIGINT NOT NULL,
    `user_id` BIGINT NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    UNIQUE (`project_issue_id`, `user_id`),
    CONSTRAINT `fk_project_issue_assignees_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_issue_assignees_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
