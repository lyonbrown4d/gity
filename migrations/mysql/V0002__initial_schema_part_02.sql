-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE `project_audit_events` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `organization_id` bigint NOT NULL,
    `event_name` varchar(255) NOT NULL,
    `action` varchar(64) NOT NULL,
    `actor_user_id` bigint NOT NULL,
    `target_type` varchar(255) NOT NULL,
    `target_id` varchar(255) NOT NULL,
    `summary` varchar(255) NOT NULL,
    `payload` text NOT NULL,
    `created_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_project_audit_events_organization_id` (`organization_id`),
    KEY `ix_project_audit_events_project_created` (`project_id`,`created_at`),
    KEY `ix_project_audit_events_actor` (`actor_user_id`),
    KEY `ix_project_audit_events_event_action` (`event_name`,`action`),
    CONSTRAINT `fk_project_audit_events_organization_id` FOREIGN KEY (`organization_id`) REFERENCES `organizations` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_audit_events_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_branch_protections` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `branch_name` varchar(255) NOT NULL,
    `rule_type` varchar(64) NOT NULL,
    `push_access_level` varchar(64) NOT NULL,
    `merge_access_level` varchar(64) NOT NULL,
    `require_merge_request` int NOT NULL,
    `require_pipeline_success` int NOT NULL,
    `allow_force_push` int NOT NULL,
    `allow_delete` int NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `project_id` (`project_id`,`branch_name`),
    CONSTRAINT `fk_project_branch_protections_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_ci_variables` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `key` varchar(255) NOT NULL,
    `value` longtext NOT NULL,
    `masked` int NOT NULL,
    `protected` int NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_ci_variables_project_key` (`project_id`,`key`),
    KEY `ix_project_ci_variables_flags` (`masked`,`protected`),
    CONSTRAINT `fk_project_ci_variables_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `project_deploy_keys` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `title` varchar(255) NOT NULL,
    `fingerprint` varchar(255) NOT NULL,
    `public_key` text NOT NULL,
    `can_push` int NOT NULL,
    `created_by_user_id` bigint NOT NULL,
    `last_used_at` timestamp(6) NULL DEFAULT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_deploy_keys_project_fingerprint` (`project_id`,`fingerprint`),
    KEY `ix_project_deploy_keys_created_by` (`created_by_user_id`),
    CONSTRAINT `fk_project_deploy_keys_created_by` FOREIGN KEY (`created_by_user_id`) REFERENCES `users` (`id`),
    CONSTRAINT `fk_project_deploy_keys_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `project_iid_counters` (
    `project_id` bigint NOT NULL,
    `counter_name` varchar(64) NOT NULL,
    `current_value` bigint NOT NULL DEFAULT '0',
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`project_id`,`counter_name`),
    CONSTRAINT `fk_project_iid_counters_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_issues` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `iid` bigint NOT NULL,
    `author_user_id` bigint NOT NULL,
    `title` varchar(255) NOT NULL,
    `description` text,
    `state` varchar(64) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_issues_project_iid` (`project_id`,`iid`),
    KEY `ix_project_issues_project_state` (`project_id`,`state`),
    KEY `ix_project_issues_author` (`author_user_id`),
    CONSTRAINT `fk_project_issues_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `fk_project_issues_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_jobs` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `kind` varchar(64) NOT NULL,
    `status` varchar(64) NOT NULL,
    `payload` longtext,
    `result` longtext,
    `attempts` int NOT NULL,
    `max_attempts` int NOT NULL,
    `run_after` datetime(6) NOT NULL,
    `locked_by` varchar(255) DEFAULT NULL,
    `locked_until` datetime(6) NOT NULL,
    `last_error` longtext,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    `started_at` datetime(6) NOT NULL,
    `finished_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_project_jobs_project_id` (`project_id`),
    KEY `ix_project_jobs_queue` (`status`,`run_after`,`kind`),
    KEY `ix_project_jobs_locked` (`locked_by`,`locked_until`),
    CONSTRAINT `fk_project_jobs_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_lfs_locks` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `owner_user_id` bigint NOT NULL,
    `path` varchar(512) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_lfs_locks_project_path` (`project_id`,`path`),
    KEY `ix_project_lfs_locks_owner` (`owner_user_id`),
    CONSTRAINT `fk_project_lfs_locks_owner_user_id` FOREIGN KEY (`owner_user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_lfs_locks_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_lfs_objects` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `oid` varchar(255) NOT NULL,
    `byte_size` bigint NOT NULL,
    `storage_key` varchar(512) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `storage_key` (`storage_key`),
    UNIQUE KEY `ux_project_lfs_objects_project_oid` (`project_id`,`oid`),
    CONSTRAINT `fk_project_lfs_objects_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_members` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `role` varchar(64) NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_members_project_user` (`project_id`,`user_id`),
    KEY `ix_project_members_user` (`user_id`),
    CONSTRAINT `fk_project_members_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_members_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `project_merge_request_approval_rules` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `target_branch` varchar(255) NOT NULL,
    `approvals_required` int NOT NULL,
    `eligible_user_ids` text NOT NULL,
    `code_owner` int NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `ix_project_mr_approval_rules_project_branch` (`project_id`,`target_branch`),
    KEY `ix_project_mr_approval_rules_code_owner` (`code_owner`),
    CONSTRAINT `fk_project_mr_approval_rules_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `project_merge_requests` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `iid` bigint NOT NULL,
    `author_user_id` bigint NOT NULL,
    `title` varchar(255) NOT NULL,
    `description` text,
    `state` varchar(64) NOT NULL,
    `source_branch` varchar(255) NOT NULL,
    `target_branch` varchar(255) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_merge_requests_project_iid` (`project_id`,`iid`),
    KEY `ix_project_merge_requests_project_state` (`project_id`,`state`),
    KEY `ix_project_merge_requests_author` (`author_user_id`),
    KEY `ix_project_merge_requests_branches` (`project_id`,`source_branch`,`target_branch`),
    CONSTRAINT `fk_project_merge_requests_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `fk_project_merge_requests_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_packages` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `type` varchar(64) NOT NULL,
    `name` varchar(255) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_packages_project_type_name` (`project_id`,`type`,`name`),
    CONSTRAINT `fk_project_packages_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
