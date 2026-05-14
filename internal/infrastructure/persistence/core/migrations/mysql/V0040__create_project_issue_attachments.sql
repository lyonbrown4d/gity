CREATE TABLE IF NOT EXISTS `project_issue_attachments` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_issue_id` BIGINT NOT NULL,
    `uploaded_by_user_id` BIGINT NOT NULL,
    `file_name` VARCHAR(255) NOT NULL,
    `content_type` VARCHAR(255),
    `byte_size` BIGINT NOT NULL,
    `storage_key` VARCHAR(512) NOT NULL UNIQUE,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_issue_attachments_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_issue_attachments_uploaded_by_user_id` FOREIGN KEY (`uploaded_by_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
