-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE `project_merge_request_comments` (
    `id` bigint NOT NULL,
    `merge_request_id` bigint NOT NULL,
    `author_user_id` bigint NOT NULL,
    `body` text NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_project_merge_request_comments_author_user_id` (`author_user_id`),
    KEY `ix_project_merge_request_comments_mr_created` (`merge_request_id`,`created_at`),
    CONSTRAINT `fk_project_merge_request_comments_author_user_id` FOREIGN KEY (`author_user_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `fk_project_merge_request_comments_merge_request_id` FOREIGN KEY (`merge_request_id`) REFERENCES `project_merge_requests` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_merge_request_participants` (
    `id` bigint NOT NULL,
    `merge_request_id` bigint NOT NULL,
    `user_id` bigint NOT NULL,
    `role` varchar(64) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `merge_request_id` (`merge_request_id`,`user_id`,`role`),
    KEY `ix_project_merge_request_participants_user` (`user_id`),
    CONSTRAINT `fk_project_merge_request_participants_merge_request_id` FOREIGN KEY (`merge_request_id`) REFERENCES `project_merge_requests` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_merge_request_participants_user_id` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_package_versions` (
    `id` bigint NOT NULL,
    `project_package_id` bigint NOT NULL,
    `version` varchar(255) NOT NULL,
    `status` varchar(64) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_project_package_versions_package_version` (`project_package_id`,`version`),
    KEY `ix_project_package_versions_package_status` (`project_package_id`,`status`),
    CONSTRAINT `fk_project_package_versions_project_package_id` FOREIGN KEY (`project_package_id`) REFERENCES `project_packages` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_pipeline_jobs` (
    `id` bigint NOT NULL,
    `project_id` bigint NOT NULL,
    `pipeline_id` bigint NOT NULL,
    `project_job_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `stage` varchar(255) NOT NULL,
    `needs` longtext,
    `image` varchar(255) DEFAULT NULL,
    `script` longtext NOT NULL,
    `artifacts` longtext,
    `sort_order` int NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `fk_project_pipeline_jobs_project_id` (`project_id`),
    KEY `fk_project_pipeline_jobs_project_job_id` (`project_job_id`),
    KEY `ix_project_pipeline_jobs_pipeline_stage_order` (`pipeline_id`,`stage`,`sort_order`),
    CONSTRAINT `fk_project_pipeline_jobs_pipeline_id` FOREIGN KEY (`pipeline_id`) REFERENCES `project_pipelines` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_pipeline_jobs_project_id` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_project_pipeline_jobs_project_job_id` FOREIGN KEY (`project_job_id`) REFERENCES `project_jobs` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `project_release_links` (
    `id` bigint NOT NULL,
    `project_release_id` bigint NOT NULL,
    `name` varchar(255) NOT NULL,
    `url` text NOT NULL,
    `link_type` varchar(64) NOT NULL,
    `created_at` timestamp(6) NOT NULL,
    `updated_at` timestamp(6) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `ix_project_release_links_release` (`project_release_id`),
    KEY `ix_project_release_links_type` (`link_type`),
    CONSTRAINT `fk_project_release_links_release_id` FOREIGN KEY (`project_release_id`) REFERENCES `project_releases` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `project_package_files` (
    `id` bigint NOT NULL,
    `project_package_version_id` bigint NOT NULL,
    `file_name` varchar(255) NOT NULL,
    `file_path` varchar(512) NOT NULL,
    `content_type` varchar(255) DEFAULT NULL,
    `byte_size` bigint NOT NULL,
    `storage_key` varchar(512) NOT NULL,
    `created_at` datetime(6) NOT NULL,
    `updated_at` datetime(6) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `storage_key` (`storage_key`),
    UNIQUE KEY `ux_project_package_files_version_path` (`project_package_version_id`,`file_path`),
    CONSTRAINT `fk_project_package_files_project_package_version_id` FOREIGN KEY (`project_package_version_id`) REFERENCES `project_package_versions` (`id`) ON DELETE CASCADE
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
