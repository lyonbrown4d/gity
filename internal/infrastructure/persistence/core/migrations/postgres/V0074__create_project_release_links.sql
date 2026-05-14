CREATE TABLE IF NOT EXISTS "project_release_links" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_release_id" BIGINT NOT NULL,
    "name" TEXT NOT NULL,
    "url" TEXT NOT NULL,
    "link_type" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_release_id") REFERENCES "project_releases" ("id") ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS "ix_project_release_links_release" ON "project_release_links" ("project_release_id");
CREATE INDEX IF NOT EXISTS "ix_project_release_links_type" ON "project_release_links" ("link_type");
