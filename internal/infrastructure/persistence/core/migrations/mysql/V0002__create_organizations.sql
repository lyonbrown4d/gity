CREATE TABLE IF NOT EXISTS `organizations` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL,
    `path_key` VARCHAR(255) NOT NULL UNIQUE,
    `full_path` VARCHAR(512) NOT NULL UNIQUE,
    `description` TEXT,
    `visibility` VARCHAR(64) NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    `updated_at` DATETIME(6) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
