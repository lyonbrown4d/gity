CREATE TABLE IF NOT EXISTS `project_merge_request_participants` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `merge_request_id` BIGINT NOT NULL,
    `user_id` BIGINT NOT NULL,
    `role` VARCHAR(64) NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    UNIQUE (`merge_request_id`, `user_id`, `role`),
    CONSTRAINT `fk_project_merge_request_participants_merge_request_id` FOREIGN KEY (`merge_request_id`) REFERENCES `project_merge_requests` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_merge_request_participants_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
