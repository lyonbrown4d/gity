-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE "project_merge_requests" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "iid" INTEGER NOT NULL,
    "author_user_id" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "state" TEXT NOT NULL,
    "source_branch" TEXT NOT NULL,
    "target_branch" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);

CREATE INDEX "ix_project_merge_requests_author" ON "project_merge_requests" ("author_user_id");

CREATE INDEX "ix_project_merge_requests_branches" ON "project_merge_requests" ("project_id", "source_branch", "target_branch");

CREATE INDEX "ix_project_merge_requests_project_state" ON "project_merge_requests" ("project_id", "state");

CREATE UNIQUE INDEX "ux_project_merge_requests_project_iid" ON "project_merge_requests" ("project_id", "iid");

CREATE TABLE "project_packages" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "type" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX "ux_project_packages_project_type_name" ON "project_packages" ("project_id", "type", "name");

CREATE TABLE "project_pipelines" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "iid" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "source" TEXT NOT NULL,
    "ref_name" TEXT,
    "commit_sha" TEXT,
    "status" TEXT NOT NULL,
    "config_source" TEXT NOT NULL,
    "config_content" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "started_at" TIMESTAMP NOT NULL,
    "finished_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_pipelines_project_ref" ON "project_pipelines" ("project_id", "ref_name");

CREATE INDEX "ix_project_pipelines_project_status" ON "project_pipelines" ("project_id", "status");

CREATE UNIQUE INDEX "ux_project_pipelines_project_iid" ON "project_pipelines" ("project_id", "iid");

CREATE TABLE "project_releases" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "tag_name" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "created_by_user_id" INTEGER NOT NULL,
    "released_at" TIMESTAMP NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("created_by_user_id") REFERENCES "users" ("id")
);

CREATE INDEX "ix_project_releases_created_by" ON "project_releases" ("created_by_user_id");

CREATE INDEX "ix_project_releases_project_released" ON "project_releases" ("project_id", "released_at");

CREATE UNIQUE INDEX "ux_project_releases_project_tag" ON "project_releases" ("project_id", "tag_name");

CREATE TABLE "project_runners" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "tags" TEXT,
    "token" TEXT NOT NULL UNIQUE,
    "status" TEXT NOT NULL,
    "active" INTEGER NOT NULL,
    "last_contact_at" TIMESTAMP NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_runners_active_contact" ON "project_runners" ("active", "last_contact_at");

CREATE INDEX "ix_project_runners_project_status" ON "project_runners" ("project_id", "status");

CREATE TABLE "project_todos" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "user_id" INTEGER NOT NULL,
    "organization_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "kind" TEXT NOT NULL,
    "state" TEXT NOT NULL,
    "target_type" TEXT NOT NULL,
    "target_id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "summary" TEXT NOT NULL,
    "action_url" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "done_at" TIMESTAMP NULL,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_todos_organization_user" ON "project_todos" ("organization_id", "user_id");

CREATE INDEX "ix_project_todos_project_user" ON "project_todos" ("project_id", "user_id");

CREATE INDEX "ix_project_todos_user_state_created" ON "project_todos" ("user_id", "state", "created_at");

CREATE UNIQUE INDEX "ux_project_todos_user_kind_target" ON "project_todos" ("user_id", "kind", "target_type", "target_id");

CREATE TABLE "project_wiki_pages" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "slug" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "content" TEXT,
    "format" TEXT NOT NULL,
    "author_user_id" INTEGER NOT NULL,
    "last_edited_by_user_id" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_id", "slug"),
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT,
    FOREIGN KEY ("last_edited_by_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);

CREATE INDEX "ix_project_wiki_pages_project_format" ON "project_wiki_pages" ("project_id", "format");

CREATE TABLE "project_issue_assignees" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_issue_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_issue_id", "user_id"),
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_issue_assignees_user" ON "project_issue_assignees" ("user_id");

CREATE TABLE "project_issue_attachments" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_issue_id" INTEGER NOT NULL,
    "uploaded_by_user_id" INTEGER NOT NULL,
    "file_name" TEXT NOT NULL,
    "content_type" TEXT,
    "byte_size" INTEGER NOT NULL,
    "storage_key" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("uploaded_by_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);

CREATE TABLE "project_issue_comments" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_issue_id" INTEGER NOT NULL,
    "author_user_id" INTEGER NOT NULL,
    "body" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("author_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT
);

CREATE INDEX "ix_project_issue_comments_issue_created" ON "project_issue_comments" ("project_issue_id", "created_at");

CREATE TABLE "project_issue_labels" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_issue_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "color" TEXT,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    UNIQUE ("project_issue_id", "name"),
    FOREIGN KEY ("project_issue_id") REFERENCES "project_issues" ("id") ON DELETE CASCADE
);

CREATE TABLE "project_job_artifacts" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "project_job_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "file_name" TEXT NOT NULL,
    "file_path" TEXT NOT NULL,
    "content_type" TEXT NOT NULL,
    "byte_size" INTEGER NOT NULL,
    "storage_key" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("project_job_id") REFERENCES "project_jobs" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_project_job_artifacts_job" ON "project_job_artifacts" ("project_job_id");
