CREATE TABLE IF NOT EXISTS "project_package_files" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_package_version_id" INTEGER NOT NULL,
    "file_name" TEXT NOT NULL,
    "file_path" TEXT NOT NULL,
    "content_type" TEXT,
    "byte_size" INTEGER NOT NULL,
    "storage_key" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_package_version_id") REFERENCES "project_package_versions" ("id") ON DELETE CASCADE
);
