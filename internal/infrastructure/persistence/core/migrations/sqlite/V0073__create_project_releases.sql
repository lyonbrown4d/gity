CREATE TABLE IF NOT EXISTS "project_releases" (
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

CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_releases_project_tag" ON "project_releases" ("project_id", "tag_name");
CREATE INDEX IF NOT EXISTS "ix_project_releases_project_released" ON "project_releases" ("project_id", "released_at");
CREATE INDEX IF NOT EXISTS "ix_project_releases_created_by" ON "project_releases" ("created_by_user_id");
