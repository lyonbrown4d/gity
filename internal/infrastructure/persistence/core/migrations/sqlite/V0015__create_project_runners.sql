CREATE TABLE IF NOT EXISTS "project_runners" (
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
