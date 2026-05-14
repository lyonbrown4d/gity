CREATE TABLE IF NOT EXISTS `project_audit_events` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `organization_id` BIGINT NOT NULL,
    `event_name` VARCHAR(255) NOT NULL,
    `action` VARCHAR(64) NOT NULL,
    `actor_user_id` BIGINT NOT NULL,
    `target_type` VARCHAR(255) NOT NULL,
    `target_id` VARCHAR(255) NOT NULL,
    `summary` VARCHAR(255) NOT NULL,
    `payload` TEXT NOT NULL,
    `created_at` DATETIME(6) NOT NULL,
    CONSTRAINT `fk_project_audit_events_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_audit_events_organization_id` FOREIGN KEY (`organization_id`) REFERENCES `organizations` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
