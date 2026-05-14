CREATE TABLE IF NOT EXISTS `project_release_links` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_release_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `url` TEXT NOT NULL,
    `link_type` VARCHAR(64) NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_release_links_release_id` FOREIGN KEY (`project_release_id`) REFERENCES `project_releases` (`id`) ON DELETE CASCADE,
    KEY `ix_project_release_links_release` (`project_release_id`),
    KEY `ix_project_release_links_type` (`link_type`)
);
