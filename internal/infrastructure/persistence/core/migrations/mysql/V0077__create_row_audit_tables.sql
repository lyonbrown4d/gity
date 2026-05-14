CREATE TABLE IF NOT EXISTS `audit_revisions` (
    `id` BIGINT NOT NULL PRIMARY KEY,
    `created_at` TIMESTAMP(6) NOT NULL,
    `actor` VARCHAR(255) NULL,
    `tenant` VARCHAR(255) NULL,
    `reason` VARCHAR(255) NULL,
    `metadata` TEXT NULL,
    KEY `ix_audit_revisions_created_at` (`created_at`),
    KEY `ix_audit_revisions_actor` (`actor`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `project_branch_protection_audits` (
    `revision_id` BIGINT NOT NULL,
    `operation` VARCHAR(32) NOT NULL,
    `project_branch_protection_id` BIGINT NOT NULL,
    `project_id` BIGINT NOT NULL,
    `branch_name` VARCHAR(255) NOT NULL,
    `rule_type` VARCHAR(64) NOT NULL,
    `push_access_level` VARCHAR(64) NOT NULL,
    `merge_access_level` VARCHAR(64) NOT NULL,
    `require_merge_request` INT NOT NULL,
    `require_pipeline_success` INT NOT NULL,
    `allow_force_push` INT NOT NULL,
    `allow_delete` INT NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_branch_protection_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`),
    KEY `ix_project_branch_protection_audits_revision` (`revision_id`),
    KEY `ix_project_branch_protection_audits_project` (`project_id`),
    KEY `ix_project_branch_protection_audits_entity` (`project_branch_protection_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `project_member_audits` (
    `revision_id` BIGINT NOT NULL,
    `operation` VARCHAR(32) NOT NULL,
    `project_member_id` BIGINT NOT NULL,
    `project_id` BIGINT NOT NULL,
    `user_id` BIGINT NOT NULL,
    `role` VARCHAR(64) NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_member_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`),
    KEY `ix_project_member_audits_revision` (`revision_id`),
    KEY `ix_project_member_audits_project` (`project_id`),
    KEY `ix_project_member_audits_entity` (`project_member_id`),
    KEY `ix_project_member_audits_user` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `project_merge_request_approval_rule_audits` (
    `revision_id` BIGINT NOT NULL,
    `operation` VARCHAR(32) NOT NULL,
    `project_merge_request_approval_rule_id` BIGINT NOT NULL,
    `project_id` BIGINT NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `target_branch` VARCHAR(255) NOT NULL,
    `approvals_required` INT NOT NULL,
    `eligible_user_ids` TEXT NOT NULL,
    `code_owner` INT NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_mr_approval_rule_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`),
    KEY `ix_project_mr_approval_rule_audits_revision` (`revision_id`),
    KEY `ix_project_mr_approval_rule_audits_project` (`project_id`),
    KEY `ix_project_mr_approval_rule_audits_entity` (`project_merge_request_approval_rule_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `project_ci_variable_audits` (
    `revision_id` BIGINT NOT NULL,
    `operation` VARCHAR(32) NOT NULL,
    `project_ci_variable_id` BIGINT NOT NULL,
    `project_id` BIGINT NOT NULL,
    `key` VARCHAR(255) NOT NULL,
    `masked` INT NOT NULL,
    `protected` INT NOT NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_ci_variable_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`),
    KEY `ix_project_ci_variable_audits_revision` (`revision_id`),
    KEY `ix_project_ci_variable_audits_project` (`project_id`),
    KEY `ix_project_ci_variable_audits_entity` (`project_ci_variable_id`),
    KEY `ix_project_ci_variable_audits_key` (`key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `project_deploy_key_audits` (
    `revision_id` BIGINT NOT NULL,
    `operation` VARCHAR(32) NOT NULL,
    `project_deploy_key_id` BIGINT NOT NULL,
    `project_id` BIGINT NOT NULL,
    `title` VARCHAR(255) NOT NULL,
    `fingerprint` VARCHAR(255) NOT NULL,
    `can_push` INT NOT NULL,
    `created_by_user_id` BIGINT NOT NULL,
    `last_used_at` TIMESTAMP(6) NULL,
    `created_at` TIMESTAMP(6) NOT NULL,
    `updated_at` TIMESTAMP(6) NOT NULL,
    CONSTRAINT `fk_project_deploy_key_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`),
    KEY `ix_project_deploy_key_audits_revision` (`revision_id`),
    KEY `ix_project_deploy_key_audits_project` (`project_id`),
    KEY `ix_project_deploy_key_audits_entity` (`project_deploy_key_id`),
    KEY `ix_project_deploy_key_audits_fingerprint` (`fingerprint`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
