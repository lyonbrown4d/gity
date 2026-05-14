CREATE TABLE IF NOT EXISTS `project_merge_request_comments` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `merge_request_id` BIGINT NOT NULL,
    `author_user_id` BIGINT NOT NULL,
    `body` TEXT NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_merge_request_comments_merge_request_id` FOREIGN KEY (`merge_request_id`) REFERENCES `project_merge_requests` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_merge_request_comments_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
