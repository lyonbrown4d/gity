CREATE TABLE IF NOT EXISTS "project_package_versions" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_package_id" BIGINT NOT NULL,
    "version" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_package_id") REFERENCES "project_packages" ("id") ON DELETE CASCADE
);
