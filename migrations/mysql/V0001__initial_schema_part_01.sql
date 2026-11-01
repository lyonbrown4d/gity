-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE `audit_revisions` (
    `id` bigint NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `actor` varchar(255) DEFAULT NULL,
    `tenant` varchar(255) DEFAULT NULL,
    `reason` varchar(255) DEFAULT NULL,
    `metadata` text,
    PRIMARY KEY (`id`),
    KEY `ix_audit_revisions_created_at` (`created_at`),
    KEY `ix_audit_revisions_actor` (`actor`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `auth_sessions` (
    `token_hash` varchar(64) NOT NULL,
    `user_id` bigint NOT NULL,
    `kind` varchar(16) NOT NULL,
    `expires_at` datetime(6) NOT NULL,
    `family` varchar(64) NOT NULL,
    PRIMARY KEY (`token_hash`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `bootstrap_claim` (
    `id` bigint NOT NULL,
    PRIMARY KEY (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `organizations` (
    `id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `path_key` varchar(255) NOT NULL,
    `full_path` varchar(512) NOT NULL,
    `description` text,
    `visibility` varchar(64) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `path_key` (`path_key`),
    UNIQUE KEY `full_path` (`full_path`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `user_passwords` (
    `user_id` bigint NOT NULL,
    `password_hash` text NOT NULL,
    PRIMARY KEY (`user_id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `users` (
    `id` bigint NOT NULL,
    `username` varchar(255) NOT NULL,
    `display_name` varchar(255) NOT NULL,
    `email` varchar(320) DEFAULT NULL,
    `is_super_admin` int NOT NULL DEFAULT '0',
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    `status` varchar(32) NOT NULL DEFAULT 'active',
    PRIMARY KEY (`id`),
    UNIQUE KEY `username` (`username`),
    UNIQUE KEY `email` (`email`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `organization_members` (
    `id` bigint NOT NULL,
    `organization_id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `role` varchar(64) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_organization_members_org_user` (`organization_id`,`user_id`),
    KEY `ix_organization_members_user` (`user_id`),
    CONSTRAINT `fk_organization_members_organization_id` FOREIGN KEY (`organization_id`) REFERENCES `organizations` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_organization_members_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_branch_protection_audits` (
    `revision_id` bigint NOT NULL,
    `operation` varchar(32) NOT NULL,
    `project_branch_protection_id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `branch_name` varchar(255) NOT NULL,
    `rule_type` varchar(64) NOT NULL,
    `push_access_level` varchar(64) NOT NULL,
    `merge_access_level` varchar(64) NOT NULL,
    `require_merge_request` int NOT NULL,
    `require_pipeline_success` int NOT NULL,
    `allow_force_push` int NOT NULL,
    `allow_delete` int NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    KEY `ix_project_branch_protection_audits_revision` (`revision_id`),
    KEY `ix_project_branch_protection_audits_project` (`project_id`),
    KEY `ix_project_branch_protection_audits_entity` (`project_branch_protection_id`),
    CONSTRAINT `fk_project_branch_protection_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_ci_variable_audits` (
    `revision_id` bigint NOT NULL,
    `operation` varchar(32) NOT NULL,
    `project_ci_variable_id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `key` varchar(255) NOT NULL,
    `masked` int NOT NULL,
    `protected` int NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    KEY `ix_project_ci_variable_audits_revision` (`revision_id`),
    KEY `ix_project_ci_variable_audits_project` (`project_id`),
    KEY `ix_project_ci_variable_audits_entity` (`project_ci_variable_id`),
    KEY `ix_project_ci_variable_audits_key` (`key`),
    CONSTRAINT `fk_project_ci_variable_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_deploy_key_audits` (
    `revision_id` bigint NOT NULL,
    `operation` varchar(32) NOT NULL,
    `project_deploy_key_id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `title` varchar(255) NOT NULL,
    `fingerprint` varchar(255) NOT NULL,
    `can_push` int NOT NULL,
    `created_by_user_id` bigint NOT NULL,
    `last_used_at` timestamp(6) NULL DEFAULT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    KEY `ix_project_deploy_key_audits_revision` (`revision_id`),
    KEY `ix_project_deploy_key_audits_project` (`project_id`),
    KEY `ix_project_deploy_key_audits_entity` (`project_deploy_key_id`),
    KEY `ix_project_deploy_key_audits_fingerprint` (`fingerprint`),
    CONSTRAINT `fk_project_deploy_key_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_member_audits` (
    `revision_id` bigint NOT NULL,
    `operation` varchar(32) NOT NULL,
    `project_member_id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `role` varchar(64) NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    KEY `ix_project_member_audits_revision` (`revision_id`),
    KEY `ix_project_member_audits_project` (`project_id`),
    KEY `ix_project_member_audits_entity` (`project_member_id`),
    KEY `ix_project_member_audits_user` (`user_id`),
    CONSTRAINT `fk_project_member_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_merge_request_approval_rule_audits` (
    `revision_id` bigint NOT NULL,
    `operation` varchar(32) NOT NULL,
    `project_merge_request_approval_rule_id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `target_branch` varchar(255) NOT NULL,
    `approvals_required` int NOT NULL,
    `eligible_user_ids` text NOT NULL,
    `code_owner` int NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    KEY `ix_project_mr_approval_rule_audits_revision` (`revision_id`),
    KEY `ix_project_mr_approval_rule_audits_project` (`project_id`),
    KEY `ix_project_mr_approval_rule_audits_entity` (`project_merge_request_approval_rule_id`),
    CONSTRAINT `fk_project_mr_approval_rule_audits_revision` FOREIGN KEY (`revision_id`) REFERENCES `audit_revisions` (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `projects` (
    `id` bigint NOT NULL,
    `organization_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `path_key` varchar(255) NOT NULL,
    `full_path` varchar(512) NOT NULL,
    `visibility` varchar(64) NOT NULL,
    `description` text,
    `default_branch` varchar(255) NOT NULL,
    `status` varchar(64) NOT NULL,
    `deleted_at` datetime(6) DEFAULT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `full_path` (`full_path`),
    KEY `ix_projects_organization_status` (`organization_id`,`status`),
    KEY `ix_projects_visibility` (`visibility`),
    CONSTRAINT `fk_projects_organization_id` FOREIGN KEY (`organization_id`) REFERENCES `organizations` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `user_access_tokens` (
    `id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `token` varchar(255) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `token` (`token`),
    KEY `fk_user_access_tokens_user_id` (`user_id`),
    CONSTRAINT `fk_user_access_tokens_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_access_tokens` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `kind` varchar(64) NOT NULL,
    `name` varchar(255) NOT NULL,
    `username` varchar(255) NOT NULL,
    `token` varchar(255) NOT NULL,
    `scopes` text NOT NULL,
    `created_by_user_id` bigint NOT NULL,
    `expires_at` timestamp(6) NULL DEFAULT NULL,
    `revoked_at` timestamp(6) NULL DEFAULT NULL,
    `last_used_at` timestamp(6) NULL DEFAULT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_access_tokens_token` (`token`),
    KEY `ix_project_access_tokens_project_kind` (`project_id`,`kind`),
    KEY `ix_project_access_tokens_username` (`username`),
    KEY `ix_project_access_tokens_created_by` (`created_by_user_id`),
    CONSTRAINT `fk_project_access_tokens_created_by` FOREIGN KEY (`created_by_user_id`) REFERENCES `users` (`id`),
    CONSTRAINT `fk_project_access_tokens_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
