CREATE TABLE IF NOT EXISTS `project_ci_variables` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `key` VARCHAR(255) NOT NULL,
    `value` TEXT NOT NULL,
    `masked` INT NOT NULL,
    `protected` INT NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_ci_variables_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    UNIQUE KEY `ux_project_ci_variables_project_key` (`project_id`, `key`),
    KEY `ix_project_ci_variables_flags` (`masked`, `protected`)
);
