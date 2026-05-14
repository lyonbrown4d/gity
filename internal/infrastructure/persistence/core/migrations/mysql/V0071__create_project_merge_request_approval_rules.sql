CREATE TABLE IF NOT EXISTS `project_merge_request_approval_rules` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `project_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `target_branch` VARCHAR(255) NOT NULL,
    `approvals_required` INT NOT NULL,
    `eligible_user_ids` TEXT NOT NULL,
    `code_owner` INT NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_mr_approval_rules_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    KEY `ix_project_mr_approval_rules_project_branch` (`project_id`, `target_branch`),
    KEY `ix_project_mr_approval_rules_code_owner` (`code_owner`)
);
