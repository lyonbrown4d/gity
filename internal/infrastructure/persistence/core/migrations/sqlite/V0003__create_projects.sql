CREATE TABLE IF NOT EXISTS "projects" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "organization_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "path_key" TEXT NOT NULL,
    "full_path" TEXT NOT NULL UNIQUE,
    "visibility" TEXT NOT NULL,
    "description" TEXT,
    "default_branch" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "deleted_at" TIMESTAMP,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE
);
