CREATE TABLE IF NOT EXISTS "project_pipelines" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "iid" BIGINT NOT NULL,
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
