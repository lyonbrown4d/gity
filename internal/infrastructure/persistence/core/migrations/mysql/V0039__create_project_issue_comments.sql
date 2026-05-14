CREATE TABLE IF NOT EXISTS `project_issue_comments` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_issue_id` BIGINT NOT NULL,
    `author_user_id` BIGINT NOT NULL,
    `body` TEXT NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_issue_comments_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_issue_comments_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
