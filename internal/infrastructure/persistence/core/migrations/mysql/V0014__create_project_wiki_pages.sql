CREATE TABLE IF NOT EXISTS `project_wiki_pages` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `slug` VARCHAR(255) NOT NULL,
    `title` VARCHAR(255) NOT NULL,
    `content` TEXT,
    `format` VARCHAR(64) NOT NULL,
    `author_user_id` BIGINT NOT NULL,
    `last_edited_by_user_id` BIGINT NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL,
    UNIQUE (`project_id`, `slug`),
    CONSTRAINT `fk_project_wiki_pages_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_wiki_pages_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `fk_project_wiki_pages_last_edited_by_user_id` FOREIGN KEY (`last_edited_by_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
