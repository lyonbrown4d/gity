-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE `project_pipelines` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `iid` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `source` varchar(64) NOT NULL,
    `ref_name` varchar(255) DEFAULT NULL,
    `commit_sha` varchar(255) DEFAULT NULL,
    `status` varchar(64) NOT NULL,
    `config_source` varchar(64) NOT NULL,
    `config_content` longtext NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    `started_at` datetime(6) NOT NULL,
    `finished_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_pipelines_project_iid` (`project_id`,`iid`),
    KEY `ix_project_pipelines_project_status` (`project_id`,`status`),
    KEY `ix_project_pipelines_project_ref` (`project_id`,`ref_name`),
    CONSTRAINT `fk_project_pipelines_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_releases` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `tag_name` varchar(255) NOT NULL,
    `name` varchar(255) NOT NULL,
    `description` text NOT NULL,
    `created_by_user_id` bigint NOT NULL,
    `released_at` timestamp(6) NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_releases_project_tag` (`project_id`,`tag_name`),
    KEY `ix_project_releases_project_released` (`project_id`,`released_at`),
    KEY `ix_project_releases_created_by` (`created_by_user_id`),
    CONSTRAINT `fk_project_releases_created_by` FOREIGN KEY (`created_by_user_id`) REFERENCES `users` (`id`),
    CONSTRAINT `fk_project_releases_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `project_runners` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `description` text,
    `tags` varchar(255) DEFAULT NULL,
    `token` varchar(255) NOT NULL,
    `status` varchar(64) NOT NULL,
    `active` int NOT NULL,
    `last_contact_at` datetime(6) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `token` (`token`),
    KEY `ix_project_runners_project_status` (`project_id`,`status`),
    KEY `ix_project_runners_active_contact` (`active`,`last_contact_at`),
    CONSTRAINT `fk_project_runners_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_todos` (
    `id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `organization_id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `kind` varchar(64) NOT NULL,
    `state` varchar(32) NOT NULL,
    `target_type` varchar(64) NOT NULL,
    `target_id` varchar(128) NOT NULL,
    `title` varchar(255) NOT NULL,
    `summary` text NOT NULL,
    `action_url` varchar(512) NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    `done_at` timestamp(6) NULL DEFAULT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_todos_user_kind_target` (`user_id`,`kind`,`target_type`,`target_id`),
    KEY `ix_project_todos_user_state_created` (`user_id`,`state`,`created_at`),
    KEY `ix_project_todos_project_user` (`project_id`,`user_id`),
    KEY `ix_project_todos_organization_user` (`organization_id`,`user_id`),
    CONSTRAINT `fk_project_todos_organization_id` FOREIGN KEY (`organization_id`) REFERENCES `organizations` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_todos_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_todos_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `project_wiki_pages` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `slug` varchar(255) NOT NULL,
    `title` varchar(255) NOT NULL,
    `content` text,
    `format` varchar(64) NOT NULL,
    `author_user_id` bigint NOT NULL,
    `last_edited_by_user_id` bigint NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `project_id` (`project_id`,`slug`),
    KEY `fk_project_wiki_pages_author_user_id` (`author_user_id`),
    KEY `fk_project_wiki_pages_last_edited_by_user_id` (`last_edited_by_user_id`),
    KEY `ix_project_wiki_pages_project_format` (`project_id`,`format`),
    CONSTRAINT `fk_project_wiki_pages_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `fk_project_wiki_pages_last_edited_by_user_id` FOREIGN KEY (`last_edited_by_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `fk_project_wiki_pages_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_issue_assignees` (
    `id` bigint NOT NULL,
    `project_issue_id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `project_issue_id` (`project_issue_id`,`user_id`),
    KEY `ix_project_issue_assignees_user` (`user_id`),
    CONSTRAINT `fk_project_issue_assignees_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_issue_assignees_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_issue_attachments` (
    `id` bigint NOT NULL,
    `project_issue_id` bigint NOT NULL,
    `uploaded_by_user_id` bigint NOT NULL,
    `file_name` varchar(255) NOT NULL,
    `content_type` varchar(255) DEFAULT NULL,
    `byte_size` bigint NOT NULL,
    `storage_key` varchar(512) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `storage_key` (`storage_key`),
    KEY `fk_project_issue_attachments_project_issue_id` (`project_issue_id`),
    KEY `fk_project_issue_attachments_uploaded_by_user_id` (`uploaded_by_user_id`),
    CONSTRAINT `fk_project_issue_attachments_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_issue_attachments_uploaded_by_user_id` FOREIGN KEY (`uploaded_by_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_issue_comments` (
    `id` bigint NOT NULL,
    `project_issue_id` bigint NOT NULL,
    `author_user_id` bigint NOT NULL,
    `body` text NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_project_issue_comments_author_user_id` (`author_user_id`),
    KEY `ix_project_issue_comments_issue_created` (`project_issue_id`,`created_at`),
    CONSTRAINT `fk_project_issue_comments_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `fk_project_issue_comments_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_issue_labels` (
    `id` bigint NOT NULL,
    `project_issue_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `color` varchar(64) DEFAULT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `project_issue_id` (`project_issue_id`,`name`),
    CONSTRAINT `fk_project_issue_labels_project_issue_id` FOREIGN KEY (`project_issue_id`) REFERENCES `project_issues` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_job_artifacts` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `project_job_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `file_name` varchar(255) NOT NULL,
    `file_path` varchar(512) NOT NULL,
    `content_type` varchar(255) NOT NULL,
    `byte_size` bigint NOT NULL,
    `storage_key` varchar(512) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_project_job_artifacts_project_id` (`project_id`),
    KEY `ix_project_job_artifacts_job` (`project_job_id`),
    CONSTRAINT `fk_project_job_artifacts_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_job_artifacts_project_job_id` FOREIGN KEY (`project_job_id`) REFERENCES `project_jobs` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_job_logs` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `project_job_id` bigint NOT NULL,
    `attempt` int NOT NULL,
    `exit_code` int NOT NULL,
    `output` longtext NOT NULL,
    `output_truncated` int NOT NULL,
    `duration_millis` bigint NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_job_logs_job_attempt` (`project_job_id`,`attempt`),
    KEY `fk_project_job_logs_project_id` (`project_id`),
    CONSTRAINT `fk_project_job_logs_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_job_logs_project_job_id` FOREIGN KEY (`project_job_id`) REFERENCES `project_jobs` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_merge_request_approvals` (
    `id` bigint NOT NULL,
    `merge_request_id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `merge_request_id` (`merge_request_id`,`user_id`),
    KEY `fk_project_merge_request_approvals_user_id` (`user_id`),
    CONSTRAINT `fk_project_merge_request_approvals_merge_request_id` FOREIGN KEY (`merge_request_id`) REFERENCES `project_merge_requests` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_merge_request_approvals_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
