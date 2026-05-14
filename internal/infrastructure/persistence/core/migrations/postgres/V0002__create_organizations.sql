CREATE TABLE IF NOT EXISTS "organizations" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "path_key" TEXT NOT NULL UNIQUE,
    "full_path" TEXT NOT NULL UNIQUE,
    "description" TEXT,
    "visibility" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL
);
